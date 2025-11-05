use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Attribute, FnArg, Pat, PatIdent};

use crate::{arg_attrs::ArgAttributes, macro_args::MacroArgs, visitor::Visitor};

/// Code generator
pub struct CodeGenerator {
    pub(crate) input_fn: syn::ItemFn,
    pub(crate) macro_args: MacroArgs,
}

impl CodeGenerator {
    /// Generate helper macros for multiline print support
    fn generate_helper_macros() -> proc_macro2::TokenStream {
        quote! {
            macro_rules! __lg_print {
                ($print_macro:ident, $level:expr, $($args:tt)*) => {
                    {
                        let __lg_formatted = format!($($args)*);
                        for __lg_line in __lg_formatted.lines() {
                            $print_macro!("{}{}", "│".repeat($level + 1), __lg_line);
                        }
                        if __lg_formatted.ends_with('\n') {
                            $print_macro!("{}", "│".repeat($level + 1));
                        }
                    }
                };
            }

            macro_rules! __lg_print_no_newline {
                ($print_macro:ident, $level:expr, $($args:tt)*) => {
                    {
                        let __lg_formatted = format!($($args)*);
                        let __lg_lines: Vec<&str> = __lg_formatted.lines().collect();
                        for (i, __lg_line) in __lg_lines.iter().enumerate() {
                            if i == 0 {
                                $print_macro!("{}{}", "│".repeat($level + 1), __lg_line);
                            } else {
                                $print_macro!("\n{}{}", "│".repeat($level + 1), __lg_line);
                            }
                        }
                        if __lg_formatted.ends_with('\n') {
                            $print_macro!("\n{}", "│".repeat($level + 1));
                        }
                    }
                };
            }
        }
    }

    /// Generate recursion depth check
    fn generate_recursion_check(&self, fn_name: &syn::Ident) -> proc_macro2::TokenStream {
        if let Some(limit) = self.macro_args.recursion_limit {
            quote! {
                if __procon_lg_depth_guard.current_depth() >= #limit {
                    panic!("Recursion limit exceeded: {} reached maximum depth of {}", stringify!(#fn_name), #limit);
                }
            }
        } else {
            quote! {}
        }
    }

    /// Generate return value output
    fn generate_return_output(_fn_return_type: &syn::ReturnType) -> proc_macro2::TokenStream {
        // New behavior: return value is never shown by default
        quote! {
            // Return value output is disabled by default
        }
    }

    /// Generate argument format expressions
    fn generate_arg_format_expressions(&self) -> Vec<proc_macro2::TokenStream> {
        let printable_args = self.extract_printable_args();

        printable_args
            .iter()
            .map(|(ident, arg_type, attrs)| {
                let format_expr = attrs.generate_format_tokens(ident, arg_type);
                let arg_name_str = ident.to_string();
                quote! {
                    if !args_str.is_empty() {
                        args_str.push_str(", ");
                    }
                    std::fmt::Write::write_fmt(&mut args_str, format_args!("{}:{}", #arg_name_str, #format_expr)).unwrap();
                }
            })
            .collect()
    }

    /// Generate complete code
    pub fn generate(&self) -> proc_macro2::TokenStream {
        let fn_name = &self.input_fn.sig.ident;
        let fn_return_type = &self.input_fn.sig.output;
        let fn_unsafety = &self.input_fn.sig.unsafety;
        let fn_generics = &self.input_fn.sig.generics;
        let fn_vis = &self.input_fn.vis;
        let mut fn_block = self.input_fn.block.clone();

        // Transform recursive calls and print-like macros
        let mut visitor = Visitor;
        visitor.visit_block_mut(&mut fn_block);

        // Extract argument information
        let outer_fn_args = self.create_outer_fn_args();

        let (impl_generics, _, where_clause) = fn_generics.split_for_impl();

        // Generate code components
        let helper_macros = Self::generate_helper_macros();
        let recursion_check = self.generate_recursion_check(fn_name);
        let return_output = Self::generate_return_output(fn_return_type);
        let arg_format_exprs = self.generate_arg_format_expressions();

        quote! {
            #fn_vis #fn_unsafety fn #fn_name #impl_generics (#outer_fn_args) #fn_return_type #where_clause {
                use procon_lg::DepthGuard;

                #helper_macros

                let __procon_lg_depth_guard = DepthGuard::new();
                #recursion_check

                let mut args_str = String::new();
                #(#arg_format_exprs)*

                if __procon_lg_depth_guard.current_depth() == 0 {
                    eprintln!("{}({})", stringify!(#fn_name), args_str);
                } else {
                    eprintln!(
                        "{}├{}({})",
                        "│".repeat(__procon_lg_depth_guard.current_depth()),
                        stringify!(#fn_name),
                        args_str
                    );
                }

                let ans = #fn_block;

                #return_output

                ans
            }
        }
    }

    /// Extract arguments for debug output with their formatting information
    fn extract_printable_args(&self) -> Vec<(&syn::Ident, &syn::Type, ArgAttributes)> {
        self.input_fn
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                        let arg_attrs = ArgAttributes::from_attrs(&pat_type.attrs);
                        if arg_attrs.should_print() {
                            Some((ident, &*pat_type.ty, arg_attrs))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Create argument list for outer function (remove only custom attributes)
    fn create_outer_fn_args(&self) -> syn::punctuated::Punctuated<FnArg, syn::Token![,]> {
        self.input_fn
            .sig
            .inputs
            .iter()
            .map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    let mut new_pat_type = pat_type.clone();
                    // Remove custom attributes (show)
                    new_pat_type.attrs.retain(|attr| !is_custom_attr(attr));
                    FnArg::Typed(new_pat_type)
                } else {
                    arg.clone()
                }
            })
            .collect()
    }
}

fn is_custom_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("show")
}

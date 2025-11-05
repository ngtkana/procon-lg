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
                            $print_macro!("{}{}", "│ ".repeat($level), __lg_line);
                        }
                        if __lg_formatted.ends_with('\n') {
                            $print_macro!("{}", "│ ".repeat($level));
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
                                $print_macro!("{}{}", "│ ".repeat($level), __lg_line);
                            } else {
                                $print_macro!("\n{}{}", "│ ".repeat($level), __lg_line);
                            }
                        }
                        if __lg_formatted.ends_with('\n') {
                            $print_macro!("\n{}", "│ ".repeat($level));
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
    fn generate_return_output(&self) -> proc_macro2::TokenStream {
        if self.macro_args.show_return {
            quote! {
                eprintln!(
                    "{}└ return: {:?}",
                    "│ ".repeat(__procon_lg_depth_guard.current_depth()),
                    ans
                );
            }
        } else {
            quote! {
                eprintln!(
                    "{}╵",
                    "│ ".repeat(__procon_lg_depth_guard.current_depth()),
                );
            }
        }
    }

    /// Generate argument format expressions
    fn generate_arg_format_expressions(&self) -> Vec<proc_macro2::TokenStream> {
        let printable_args = self.extract_printable_args();

        printable_args
            .iter()
            .map(|(ident_token, type_token, attrs)| {
                let format_expr = if let Some(formatter) = attrs.get_custom_formatter() {
                    quote! {
                        (|x: &#type_token| #formatter)(&#ident_token)
                    }
                } else {
                    quote! {
                        format!("{:?}", #ident_token)
                    }
                };
                let arg_name_str = ident_token.to_string();
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
        let return_output = self.generate_return_output();
        let arg_format_exprs = self.generate_arg_format_expressions();

        quote! {
            #fn_vis #fn_unsafety fn #fn_name #impl_generics (#outer_fn_args) #fn_return_type #where_clause {
                use procon_lg::DepthGuard;

                #helper_macros

                let __procon_lg_depth_guard = DepthGuard::new();
                #recursion_check

                let mut args_str = String::new();
                #(#arg_format_exprs)*

                eprintln!(
                    "{}{}({})",
                    "│ ".repeat(__procon_lg_depth_guard.current_depth()),
                    stringify!(#fn_name),
                    args_str
                );

                let ans = #fn_block;

                #return_output

                ans
            }
        }
    }

    /// Extract arguments for debug output with their formatting information
    fn extract_printable_args(
        &self,
    ) -> Vec<(
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        ArgAttributes,
    )> {
        self.input_fn
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Receiver(receiver) => {
                    let arg_attrs = ArgAttributes::from_attrs(&receiver.attrs);
                    if arg_attrs.should_print() {
                        let self_token = quote! { self };
                        let self_type = if receiver.mutability.is_some() {
                            if receiver.reference.is_some() {
                                quote! { &mut Self }
                            } else {
                                quote! { Self }
                            }
                        } else if receiver.reference.is_some() {
                            quote! { &Self }
                        } else {
                            quote! { Self }
                        };
                        Some((self_token, self_type, arg_attrs))
                    } else {
                        None
                    }
                }
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                        let arg_attrs = ArgAttributes::from_attrs(&pat_type.attrs);
                        if arg_attrs.should_print() {
                            let ident_token = quote! { #ident };
                            let type_ref = &*pat_type.ty;
                            let type_token = quote! { #type_ref };
                            Some((ident_token, type_token, arg_attrs))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
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
            .map(|arg| match arg {
                FnArg::Receiver(receiver) => {
                    let mut new_receiver = receiver.clone();
                    // Remove custom attributes (show)
                    new_receiver.attrs.retain(|attr| !is_custom_attr(attr));
                    FnArg::Receiver(new_receiver)
                }
                FnArg::Typed(pat_type) => {
                    let mut new_pat_type = pat_type.clone();
                    // Remove custom attributes (show)
                    new_pat_type.attrs.retain(|attr| !is_custom_attr(attr));
                    FnArg::Typed(new_pat_type)
                }
            })
            .collect()
    }
}

fn is_custom_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("show")
}

use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Attribute, FnArg, Pat, PatIdent};
use syn::{ReturnType, Type};

use crate::{arg_attrs::ArgAttributes, args::MacroArgs, visitor::Visitor};

/// Code generator
pub struct CodeGenerator {
    pub(crate) input_fn: syn::ItemFn,
    pub(crate) macro_args: MacroArgs,
}

impl CodeGenerator {
    /// Generate complete code
    pub fn generate(&self) -> proc_macro2::TokenStream {
        let fn_name = &self.input_fn.sig.ident;
        let fn_return_type = &self.input_fn.sig.output;
        let fn_unsafety = &self.input_fn.sig.unsafety;
        let fn_generics = &self.input_fn.sig.generics;
        let fn_vis = &self.input_fn.vis;
        let mut fn_block = self.input_fn.block.clone();

        // Do the below:
        // 1. Transform recursive calls
        // 2. Transform print-like macros
        let mut visitor = Visitor { fn_name };
        visitor.visit_block_mut(&mut fn_block);

        let show_return = !self.macro_args.no_return && !is_unit_return_type(fn_return_type);

        let printable_args = self.extract_printable_args();
        let all_arg_names = self.extract_all_arg_names();
        let outer_fn_args = self.create_outer_fn_args();
        let inner_fn_args = self.create_inner_fn_args();

        let (impl_generics, _, where_clause) = fn_generics.split_for_impl();

        let return_output = if show_return {
            quote! {
                eprintln!(
                    "{}└ {:?}",
                    "│".repeat(__lg_recur_level),
                    ans
                );
            }
        } else {
            quote! {
                // Return value output is disabled
            }
        };

        // Generate custom format expressions
        let arg_format_exprs: Vec<_> = printable_args
            .iter()
            .map(|(ident, arg_type, attrs)| {
                let format_expr = attrs.generate_format_tokens(ident, arg_type);
                if attrs.should_hide_name() {
                    quote! {
                        if !args_str.is_empty() {
                            args_str.push_str(", ");
                        }
                        args_str.push_str(&format!("{}", #format_expr));
                    }
                } else {
                    let arg_name_str = ident.to_string();
                    quote! {
                        if !args_str.is_empty() {
                            args_str.push_str(", ");
                        }
                        args_str.push_str(&format!("{}:{}", #arg_name_str, #format_expr));
                    }
                }
            })
            .collect();

        quote! {
            #fn_vis #fn_unsafety fn #fn_name #impl_generics (#outer_fn_args) #fn_return_type #where_clause {
                #fn_unsafety fn __procon_lg_recurse #impl_generics (#inner_fn_args, __lg_recur_level: usize) #fn_return_type #where_clause {
                    let mut args_str = String::new();
                    #(#arg_format_exprs)*

                    if __lg_recur_level == 0 {
                        eprintln!("{}({})", stringify!(#fn_name), args_str);
                    } else {
                        eprintln!(
                            "{}├┬({})",
                            "│".repeat(__lg_recur_level - 1),
                            args_str
                        );
                    }

                    let ans = #fn_block;

                    #return_output

                    ans
                }
                __procon_lg_recurse(#(#all_arg_names),*, 0)
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
                        let arg_attrs = ArgAttributes::from_attrs(&pat_type.attrs).ok()?;
                        if arg_attrs.should_include_in_debug() {
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

    /// Extract all argument names
    fn extract_all_arg_names(&self) -> Vec<&syn::Ident> {
        self.input_fn
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                        Some(ident)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Create argument list for outer function (remove mut keyword and custom attributes)
    fn create_outer_fn_args(&self) -> syn::punctuated::Punctuated<FnArg, syn::Token![,]> {
        self.input_fn
            .sig
            .inputs
            .iter()
            .map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    let mut new_pat_type = pat_type.clone();
                    if let Pat::Ident(ref mut pat_ident) = *new_pat_type.pat {
                        pat_ident.mutability = None; // Remove mut keyword
                    }
                    // Remove custom attributes (no_debug, fmt)
                    new_pat_type.attrs.retain(|attr| !is_custom_attr(attr));
                    FnArg::Typed(new_pat_type)
                } else {
                    arg.clone()
                }
            })
            .collect()
    }

    /// Create argument list for `__procon_lg_recurse` function (remove only custom
    /// attributes)proconlg
    fn create_inner_fn_args(&self) -> syn::punctuated::Punctuated<FnArg, syn::Token![,]> {
        self.input_fn
            .sig
            .inputs
            .iter()
            .map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    let mut new_pat_type = pat_type.clone();
                    // Remove custom attributes (no_debug, fmt)
                    new_pat_type.attrs.retain(|attr| !is_custom_attr(attr));
                    FnArg::Typed(new_pat_type)
                } else {
                    arg.clone()
                }
            })
            .collect()
    }
}

fn is_unit_return_type(return_type: &ReturnType) -> bool {
    match return_type {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => {
            if let Type::Tuple(tuple) = &**ty {
                tuple.elems.is_empty()
            } else {
                false
            }
        }
    }
}

fn is_custom_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("no_debug")
        || attr.path().is_ident("fmt")
        || attr.path().is_ident("no_name")
}

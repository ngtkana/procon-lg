use quote::quote;
use syn::{FnArg, Pat, PatIdent};
use crate::args::MacroArgs;
use crate::attributes::has_no_debug_attr;
use crate::utils::is_unit_return_type;

/// Code generator
pub struct CodeGenerator {
    input_fn: syn::ItemFn,
    macro_args: MacroArgs,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new(input_fn: syn::ItemFn, macro_args: MacroArgs) -> Self {
        Self { input_fn, macro_args }
    }

    /// Generate complete code
    pub fn generate(&self) -> proc_macro2::TokenStream {
        let fn_name = &self.input_fn.sig.ident;
        let fn_return_type = &self.input_fn.sig.output;
        let mut fn_block = self.input_fn.block.clone();

        // Execute AST transformation
        crate::visitor::Visitor::transform_block(fn_name.clone(), &mut fn_block);

        let is_unit_return = is_unit_return_type(fn_return_type);
        let show_return = !self.macro_args.has_no_return() && !is_unit_return;

        let printable_args = self.extract_printable_args();
        let all_arg_names = self.extract_all_arg_names();
        let outer_fn_args = self.create_outer_fn_args();
        let inner_fn_args = self.create_inner_fn_args();

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

        quote! {
            fn #fn_name(#outer_fn_args) #fn_return_type {
                fn inner(#inner_fn_args, __lg_recur_level: usize) #fn_return_type {
                    let mut args_str = String::new();
                    #(
                        if !args_str.is_empty() {
                            args_str.push_str(", ");
                        }
                        args_str.push_str(&format!("{}={:?}", stringify!(#printable_args), #printable_args));
                    )*

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
                inner(#(#all_arg_names),*, 0)
            }
        }
    }

    /// Extract arguments for debug output
    fn extract_printable_args(&self) -> Vec<&syn::Ident> {
        self.input_fn.sig.inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                        if !has_no_debug_attr(&pat_type.attrs) {
                            Some(ident)
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
        self.input_fn.sig.inputs
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

    /// Create argument list for outer function (remove mut keyword and #[no_debug] attributes)
    fn create_outer_fn_args(&self) -> syn::punctuated::Punctuated<FnArg, syn::Token![,]> {
        self.input_fn.sig.inputs
            .iter()
            .map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    let mut new_pat_type = pat_type.clone();
                    if let Pat::Ident(ref mut pat_ident) = *new_pat_type.pat {
                        pat_ident.mutability = None; // Remove mut keyword
                    }
                    // Remove #[no_debug] attributes
                    new_pat_type.attrs.retain(|attr| !has_no_debug_attr(&[attr.clone()]));
                    FnArg::Typed(new_pat_type)
                } else {
                    arg.clone()
                }
            })
            .collect()
    }

    /// Create argument list for inner function (remove only #[no_debug] attributes)
    fn create_inner_fn_args(&self) -> syn::punctuated::Punctuated<FnArg, syn::Token![,]> {
        self.input_fn.sig.inputs
            .iter()
            .map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    let mut new_pat_type = pat_type.clone();
                    // Remove #[no_debug] attributes
                    new_pat_type.attrs.retain(|attr| !has_no_debug_attr(&[attr.clone()]));
                    FnArg::Typed(new_pat_type)
                } else {
                    arg.clone()
                }
            })
            .collect()
    }
}

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprCall, FnArg, ItemFn, Pat, parse_macro_input, visit_mut::VisitMut};

struct RecursionTransformer {
    fn_name: syn::Ident,
}

impl VisitMut for RecursionTransformer {
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        if let Expr::Path(path) = &*call.func
            && path.path.is_ident(&self.fn_name)
        {
            let level_arg = syn::parse_quote!(__lg_recur_level + 1);
            call.args.push(level_arg);

            let inner_path: syn::ExprPath = syn::parse_quote!(inner);
            call.func = Box::new(Expr::Path(inner_path));
        }

        syn::visit_mut::visit_expr_call_mut(self, call);
    }
}

#[proc_macro_attribute]
pub fn lg_recur(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_args = &input_fn.sig.inputs;
    let fn_return_type = &input_fn.sig.output;
    let mut fn_block = input_fn.block.clone();

    let arg_names: Vec<_> = fn_args
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some(&pat_ident.ident)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Create outer function args without mut keywords
    let outer_fn_args: syn::punctuated::Punctuated<FnArg, syn::Token![,]> = fn_args
        .iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                let mut new_pat_type = pat_type.clone();
                if let Pat::Ident(ref mut pat_ident) = *new_pat_type.pat {
                    pat_ident.mutability = None; // Remove mut keyword
                }
                FnArg::Typed(new_pat_type)
            } else {
                arg.clone()
            }
        })
        .collect();

    let mut transformer = RecursionTransformer {
        fn_name: fn_name.clone(),
    };
    transformer.visit_block_mut(&mut fn_block);

    let expanded = quote! {
        fn #fn_name(#outer_fn_args) #fn_return_type {
            fn inner(#fn_args, __lg_recur_level: usize) #fn_return_type {
                let mut args_str = String::new();
                #(
                    if !args_str.is_empty() {
                        args_str.push_str(", ");
                    }
                    args_str.push_str(&format!("{}={:?}", stringify!(#arg_names), #arg_names));
                )*

                if __lg_recur_level == 0 {
                    eprintln!("{}({})", stringify!(#fn_name), args_str);
                } else {
                    eprintln!("{}", "│".repeat(__lg_recur_level));
                    eprintln!(
                        "{}┬({})",
                        "│".repeat(__lg_recur_level - 1) + "├",
                        args_str
                    );
                }

                let ans = #fn_block;

                ans
            }
            inner(#(#arg_names),*, 0)
        }
    };
    TokenStream::from(expanded)
}

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, ExprCall, FnArg, ItemFn, Pat, parse_macro_input, parse::Parse, 
    visit_mut::{self, VisitMut}, Attribute,
};

#[derive(Default)]
struct MacroArgs {
    no_return: bool,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = MacroArgs::default();
        
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            
            match ident.to_string().as_str() {
                "no_return" => args.no_return = true,
                _ => return Err(syn::Error::new(ident.span(), "unknown argument")),
            }
            
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }
        
        Ok(args)
    }
}

fn has_no_print_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("no_print")
    })
}

struct Visitor {
    fn_name: syn::Ident,
}

impl VisitMut for Visitor {
    // Converts recursive calls
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

    // Converts print-like macros
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        let path = mac.path.to_token_stream().to_string();
        let tokens = &mac.tokens;
        match path.as_str() {
            "println" => {
                if tokens.is_empty() {
                    *mac = syn::parse_quote! {
                        println!("{}", "│".repeat(__lg_recur_level + 1))
                    };
                } else {
                    *mac = syn::parse_quote! {
                        println!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
                    };
                }
            }
            "eprintln" => {
                if tokens.is_empty() {
                    *mac = syn::parse_quote! {
                        eprintln!("{}", "│".repeat(__lg_recur_level + 1))
                    };
                } else {
                    *mac = syn::parse_quote! {
                        eprintln!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
                    };
                }
            }
            "print" => {
                if tokens.is_empty() {
                    *mac = syn::parse_quote! {
                        print!("{}", "│".repeat(__lg_recur_level + 1))
                    };
                } else {
                    *mac = syn::parse_quote! {
                        print!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
                    };
                }
            }
            "eprint" => {
                if tokens.is_empty() {
                    *mac = syn::parse_quote! {
                        eprint!("{}", "│".repeat(__lg_recur_level + 1))
                    };
                } else {
                    *mac = syn::parse_quote! {
                        eprint!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
                    };
                }
            }
            _ => {}
        }
        visit_mut::visit_macro_mut(self, mac);
    }
}

#[proc_macro_attribute]
pub fn lg_recur(attr: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = if attr.is_empty() {
        MacroArgs::default()
    } else {
        parse_macro_input!(attr as MacroArgs)
    };
    
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_args = &input_fn.sig.inputs;
    let fn_return_type = &input_fn.sig.output;
    let mut fn_block = input_fn.block.clone();

    // Check if return type is unit type ()
    let is_unit_return = match fn_return_type {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => {
            if let syn::Type::Tuple(tuple) = &**ty {
                tuple.elems.is_empty()
            } else {
                false
            }
        }
    };

    // Filter arguments and their names based on #[no_print] attribute
    let printable_args: Vec<_> = fn_args
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    if !has_no_print_attr(&pat_type.attrs) {
                        Some(&pat_ident.ident)
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
        .collect();

    let all_arg_names: Vec<_> = fn_args
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

    // Create outer function args without mut keywords and #[no_print] attributes
    let outer_fn_args: syn::punctuated::Punctuated<FnArg, syn::Token![,]> = fn_args
        .iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                let mut new_pat_type = pat_type.clone();
                if let Pat::Ident(ref mut pat_ident) = *new_pat_type.pat {
                    pat_ident.mutability = None; // Remove mut keyword of outer-fn
                }
                // Remove #[no_print] attributes from outer function
                new_pat_type.attrs.retain(|attr| !has_no_print_attr(&[attr.clone()]));
                FnArg::Typed(new_pat_type)
            } else {
                arg.clone()
            }
        })
        .collect();

    // Clean inner function args (remove #[no_print] attributes)
    let inner_fn_args: syn::punctuated::Punctuated<FnArg, syn::Token![,]> = fn_args
        .iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                let mut new_pat_type = pat_type.clone();
                // Remove #[no_print] attributes from inner function
                new_pat_type.attrs.retain(|attr| !has_no_print_attr(&[attr.clone()]));
                FnArg::Typed(new_pat_type)
            } else {
                arg.clone()
            }
        })
        .collect();

    Visitor {
        fn_name: fn_name.clone(),
    }
    .visit_block_mut(&mut fn_block);

    // Determine if we should show return value
    let show_return = !macro_args.no_return && !is_unit_return;

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

    let expanded = quote! {
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
    };
    TokenStream::from(expanded)
}

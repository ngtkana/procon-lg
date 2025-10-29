use quote::ToTokens;
use syn::{
    Expr, ExprCall, ExprPath,
    visit_mut::{self, VisitMut},
};

/// Visitor for AST transformation
///
/// Call only [`Visitor::visit_block_mut`]!
pub struct Visitor<'a> {
    pub(crate) fn_name: &'a syn::Ident,
}

impl VisitMut for Visitor<'_> {
    /// Transform recursive calls
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        if let Expr::Path(path) = &*call.func
            && path.path.is_ident(self.fn_name)
        {
            let level_arg = syn::parse_quote!(__lg_recur_level + 1);
            call.args.push(level_arg);

            let inner_path: ExprPath = syn::parse_quote!(__procon_lg_recurse);
            call.func = Box::new(Expr::Path(inner_path));
        }

        syn::visit_mut::visit_expr_call_mut(self, call);
    }

    /// Transform print-like macros
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        let path = mac.path.to_token_stream().to_string();
        let tokens = &mac.tokens;

        match path.as_str() {
            "println" => {
                *mac = transform_println_macro(tokens);
            }
            "eprintln" => {
                *mac = transform_eprintln_macro(tokens);
            }
            "print" => {
                *mac = transform_print_macro(tokens);
            }
            "eprint" => {
                *mac = transform_eprint_macro(tokens);
            }
            _ => {}
        }
        visit_mut::visit_macro_mut(self, mac);
    }
}

/// Transform `println!` macro
fn transform_println_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            println!("{}", "│".repeat(__lg_recur_level + 1))
        }
    } else {
        syn::parse_quote! {
            println!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
        }
    }
}

/// Transform `eprintln!` macro
fn transform_eprintln_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            eprintln!("{}", "│".repeat(__lg_recur_level + 1))
        }
    } else {
        syn::parse_quote! {
            eprintln!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
        }
    }
}

/// Transform `print!` macro
fn transform_print_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            print!("{}", "│".repeat(__lg_recur_level + 1))
        }
    } else {
        syn::parse_quote! {
            print!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
        }
    }
}

/// Transform `eprint!` macro
fn transform_eprint_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            eprint!("{}", "│".repeat(__lg_recur_level + 1))
        }
    } else {
        syn::parse_quote! {
            eprint!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!(#tokens))
        }
    }
}

impl Visitor<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{Block, parse_quote};

    #[test]
    fn test_recursive_call() {
        let mut visitor = Visitor {
            fn_name: &parse_quote!(fib),
        };

        let mut block: Block = parse_quote! {
            {
                if n <= 1 {
                    1
                } else {
                    fib(n - 1) + fib(n - 2)
                }
            }
        };

        visitor.visit_block_mut(&mut block);

        let expected: Block = parse_quote! {
            {
                if n <= 1 {
                    1
                } else {
                    __procon_lg_recurse(n - 1, __lg_recur_level + 1) + __procon_lg_recurse(n - 2, __lg_recur_level + 1)
                }
            }
        };

        assert_eq!(quote!(#block).to_string(), quote!(#expected).to_string());
    }

    #[test]
    fn test_println() {
        let mut visitor = Visitor {
            fn_name: &parse_quote!(test_fn),
        };

        let mut block: Block = parse_quote! {
            {
                println!("computing value for {}", n);
                let result = 42;
                println!();
                result
            }
        };

        visitor.visit_block_mut(&mut block);

        let expected: Block = parse_quote! {
            {
                println!("{}{}", "│".repeat(__lg_recur_level + 1), format_args!("computing value for {}", n));
                let result = 42;
                println!("{}", "│".repeat(__lg_recur_level + 1));
                result
            }
        };

        assert_eq!(quote!(#block).to_string(), quote!(#expected).to_string());
    }
}

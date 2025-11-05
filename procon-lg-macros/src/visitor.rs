use quote::ToTokens;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprReturn};

/// Visitor for AST transformation
///
/// Call only [`Visitor::visit_block_mut`]!
pub struct Visitor;

impl VisitMut for Visitor {
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

    /// Transform return expressions to include logging
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // First visit children to avoid infinite recursion
        visit_mut::visit_expr_mut(self, expr);

        // Then transform this expression if it's a return
        if let Expr::Return(expr_return) = expr {
            *expr = transform_return_expr(expr_return);
        }
    }
}

/// Transform `println!` macro
fn transform_println_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            println!("{}", "│".repeat(__procon_lg_depth_guard.current_depth() + 1))
        }
    } else {
        syn::parse_quote! {
            __lg_print!(println, __procon_lg_depth_guard.current_depth(), #tokens)
        }
    }
}

/// Transform `eprintln!` macro
fn transform_eprintln_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            eprintln!("{}", "│".repeat(__procon_lg_depth_guard.current_depth() + 1))
        }
    } else {
        syn::parse_quote! {
            __lg_print!(eprintln, __procon_lg_depth_guard.current_depth(), #tokens)
        }
    }
}

/// Transform `print!` macro
fn transform_print_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            print!("{}", "│".repeat(__procon_lg_depth_guard.current_depth() + 1))
        }
    } else {
        syn::parse_quote! {
            __lg_print_no_newline!(print, __procon_lg_depth_guard.current_depth(), #tokens)
        }
    }
}

/// Transform `eprint!` macro
fn transform_eprint_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            eprint!("{}", "│".repeat(__procon_lg_depth_guard.current_depth() + 1))
        }
    } else {
        syn::parse_quote! {
            __lg_print_no_newline!(eprint, __procon_lg_depth_guard.current_depth(), #tokens)
        }
    }
}

/// Transform return expression to include logging
fn transform_return_expr(expr_return: &ExprReturn) -> Expr {
    match &expr_return.expr {
        Some(return_expr) => {
            syn::parse_quote! {
                {
                    let __lg_return_val = #return_expr;
                    eprintln!(
                        "{}└return: {:?}",
                        "│".repeat(__procon_lg_depth_guard.current_depth()),
                        __lg_return_val
                    );
                    return __lg_return_val;
                }
            }
        }
        None => {
            syn::parse_quote! {
                {
                    eprintln!(
                        "{}└return",
                        "│".repeat(__procon_lg_depth_guard.current_depth())
                    );
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{parse_quote, Block};

    #[test]
    fn test_println() {
        let mut visitor = Visitor;

        let mut block: Block = parse_quote! {
            {
                println!("line1\nline2\nline3");
            }
        };

        visitor.visit_block_mut(&mut block);

        let expected: Block = parse_quote! {
            {
                __lg_print!(println, __procon_lg_depth_guard.current_depth(), "line1\nline2\nline3");
            }
        };

        assert_eq!(quote!(#block).to_string(), quote!(#expected).to_string());
    }
}

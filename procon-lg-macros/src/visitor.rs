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

        if path.as_str() == "eprintln" {
            *mac = transform_eprintln_macro(tokens);
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

/// Transform `eprintln!` macro
fn transform_eprintln_macro(tokens: &proc_macro2::TokenStream) -> syn::Macro {
    if tokens.is_empty() {
        syn::parse_quote! {
            eprintln!("{}", "│ ".repeat(__procon_lg_depth_guard.current_depth() + 1))
        }
    } else {
        syn::parse_quote! {
            __lg_print!(eprintln, __procon_lg_depth_guard.current_depth() + 1, #tokens)
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
                        "{}└ return: {:?}",
                        "│ ".repeat(__procon_lg_depth_guard.current_depth()),
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
                        "{}└ return",
                        "│ ".repeat(__procon_lg_depth_guard.current_depth())
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
                eprintln!("line1\nline2\nline3");
            }
        };

        visitor.visit_block_mut(&mut block);

        let expected: Block = parse_quote! {
            {
                __lg_print!(eprintln, __procon_lg_depth_guard.current_depth() + 1, "line1\nline2\nline3");
            }
        };

        assert_eq!(quote!(#block).to_string(), quote!(#expected).to_string());
    }
}

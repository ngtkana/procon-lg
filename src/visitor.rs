use quote::ToTokens;
use syn::{
    Expr, ExprCall, ExprPath,
    visit_mut::{self, VisitMut},
};

/// Visitor for AST transformation
pub struct Visitor {
    fn_name: syn::Ident,
}

impl Visitor {
    /// Create a new visitor
    pub fn new(fn_name: syn::Ident) -> Self {
        Self { fn_name }
    }

    /// Transform a function block
    pub fn transform_block(fn_name: syn::Ident, block: &mut syn::Block) {
        let mut visitor = Self::new(fn_name);
        visitor.visit_block_mut(block);
    }
}

impl VisitMut for Visitor {
    /// Transform recursive calls
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        if let Expr::Path(path) = &*call.func
            && path.path.is_ident(&self.fn_name)
        {
            let level_arg = syn::parse_quote!(__lg_recur_level + 1);
            call.args.push(level_arg);

            let inner_path: ExprPath = syn::parse_quote!(inner);
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
                *mac = self.transform_println_macro(tokens);
            }
            "eprintln" => {
                *mac = self.transform_eprintln_macro(tokens);
            }
            "print" => {
                *mac = self.transform_print_macro(tokens);
            }
            "eprint" => {
                *mac = self.transform_eprint_macro(tokens);
            }
            _ => {}
        }
        visit_mut::visit_macro_mut(self, mac);
    }
}

impl Visitor {
    /// Transform `println!` macro
    fn transform_println_macro(&self, tokens: &proc_macro2::TokenStream) -> syn::Macro {
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
    fn transform_eprintln_macro(&self, tokens: &proc_macro2::TokenStream) -> syn::Macro {
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
    fn transform_print_macro(&self, tokens: &proc_macro2::TokenStream) -> syn::Macro {
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
    fn transform_eprint_macro(&self, tokens: &proc_macro2::TokenStream) -> syn::Macro {
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
}

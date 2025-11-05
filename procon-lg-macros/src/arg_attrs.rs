use proc_macro2::TokenStream;
use syn::{Attribute, Expr};

/// Represents different types of argument attributes
#[derive(Clone)]
pub enum ArgAttribute {
    /// #\[fmt\] - Include in debug output with default formatting
    Fmt,
    /// #\[fmt(expr)\] - Include in debug output with custom formatter
    FmtWithExpression { formatter: Expr },
}

/// Parsed argument attributes
pub struct ArgAttributes {
    pub attrs: Vec<ArgAttribute>,
}

impl ArgAttributes {
    /// Create ArgAttributes from attribute list
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        let mut parsed_attrs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("fmt") {
                if let Ok(formatter) = attr.parse_args::<Expr>() {
                    parsed_attrs.push(ArgAttribute::FmtWithExpression { formatter });
                } else {
                    parsed_attrs.push(ArgAttribute::Fmt);
                }
            }
        }

        ArgAttributes {
            attrs: parsed_attrs,
        }
    }

    /// Check if this argument should be printed
    pub fn should_print(&self) -> bool {
        !self.attrs.is_empty()
    }

    /// Get custom formatter (first one found)
    pub fn get_custom_formatter(&self) -> Option<&Expr> {
        self.attrs.iter().find_map(|attr| {
            if let ArgAttribute::FmtWithExpression { formatter } = attr {
                Some(formatter)
            } else {
                None
            }
        })
    }

    /// Generate format tokens for this argument
    pub fn generate_format_tokens(
        &self,
        arg_name: &syn::Ident,
        arg_type: &syn::Type,
    ) -> TokenStream {
        if let Some(formatter) = self.get_custom_formatter() {
            // Create a closure that wraps the formatter expression
            // The closure parameter should be a reference to the type
            quote::quote! {
                (|#arg_name: &#arg_type| #formatter)(&#arg_name)
            }
        } else {
            quote::quote! {
                format!("{:?}", #arg_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_fmt_with_expression() {
        let attr: Attribute = parse_quote!(#[fmt(format!("0x{:x}", value))]);
        let attrs = ArgAttributes::from_attrs(&[attr]);
        let value_: syn::Ident = parse_quote!(value);
        let value_type: syn::Type = parse_quote!(i32);
        let result = attrs.generate_format_tokens(&value_, &value_type);
        let expected = quote::quote! {
            (|value: &i32| format!("0x{:x}", value))(&value)
        };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_fmt_with_field_access() {
        let attr: Attribute = parse_quote!(#[fmt(node.key)]);
        let attrs = ArgAttributes::from_attrs(&[attr]);
        let node_: syn::Ident = parse_quote!(node);
        let node_type: syn::Type = parse_quote!(&Node);
        let result = attrs.generate_format_tokens(&node_, &node_type);
        let expected = quote::quote! {
            (|node: & & Node| node.key)(&node)
        };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_basic_fmt() {
        let attr: Attribute = parse_quote!(#[fmt]);
        let attrs = ArgAttributes::from_attrs(&[attr]);
        let value_: syn::Ident = parse_quote!(value);
        let value_type: syn::Type = parse_quote!(i32);
        let result = attrs.generate_format_tokens(&value_, &value_type);
        let expected: TokenStream = quote::quote! {
            format!("{:?}", value)
        };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_no_fmt() {
        let attrs = ArgAttributes::from_attrs(&[]);
        assert!(!attrs.should_print());
    }
}

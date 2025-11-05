use proc_macro2::TokenStream;
use syn::{Attribute, Expr, Result};

/// Represents different types of argument attributes
#[derive(Clone)]
pub enum ArgAttribute {
    /// #\[no_debug]] - Exclude from debug output
    NoDebug,
    /// #\[fmt(closure-impl)\] - Use custom formatter
    Fmt { formatter: Expr },
    /// #\[no_name\] - Don't show argument name (e.g., "arg=" part)
    NoName,
}

/// Parsed argument attributes
pub struct ArgAttributes {
    pub attrs: Vec<ArgAttribute>,
}

impl ArgAttributes {
    /// Create ArgAttributes from attribute list
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut parsed_attrs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("no_debug") {
                parsed_attrs.push(ArgAttribute::NoDebug);
            } else if attr.path().is_ident("fmt") {
                let formatter = attr.parse_args::<Expr>()?;
                parsed_attrs.push(ArgAttribute::Fmt { formatter });
            } else if attr.path().is_ident("no_name") {
                parsed_attrs.push(ArgAttribute::NoName);
            }
        }

        Ok(ArgAttributes {
            attrs: parsed_attrs,
        })
    }

    /// Check if this argument should be included in debug output
    pub fn should_include_in_debug(&self) -> bool {
        !self
            .attrs
            .iter()
            .any(|attr| matches!(attr, ArgAttribute::NoDebug))
    }

    /// Check if argument name should be hidden
    pub fn should_hide_name(&self) -> bool {
        self.attrs
            .iter()
            .any(|attr| matches!(attr, ArgAttribute::NoName))
    }

    /// Get custom formatter (first one found)
    pub fn get_custom_formatter(&self) -> Option<&Expr> {
        self.attrs.iter().find_map(|attr| {
            if let ArgAttribute::Fmt { formatter } = attr {
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
        let attrs = ArgAttributes::from_attrs(&[attr]).unwrap();
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
        let attrs = ArgAttributes::from_attrs(&[attr]).unwrap();
        let node_: syn::Ident = parse_quote!(node);
        let node_type: syn::Type = parse_quote!(&Node);
        let result = attrs.generate_format_tokens(&node_, &node_type);
        let expected = quote::quote! {
            (|node: & & Node| node.key)(&node)
        };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_no_fmt() {
        let attrs = ArgAttributes::from_attrs(&[]).unwrap();
        let value_: syn::Ident = parse_quote!(value);
        let value_type: syn::Type = parse_quote!(i32);
        let result = attrs.generate_format_tokens(&value_, &value_type);
        let expected: TokenStream = quote::quote! {
            format!("{:?}", value)
        };
        assert_eq!(result.to_string(), expected.to_string());
    }
}

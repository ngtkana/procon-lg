use proc_macro2::TokenStream;
use syn::{Attribute, Expr, Result};

/// Represents different types of argument attributes
#[derive(Clone)]
pub enum ArgAttribute {
    /// #\[no_debug]] - Exclude from debug output
    NoDebug,
    /// #\[fmt(closure)\] - Use custom formatter
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
    pub fn generate_format_tokens(&self, arg_name: &syn::Ident) -> TokenStream {
        if let Some(formatter) = self.get_custom_formatter() {
            quote::quote! {
                (#formatter)(&#arg_name)
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
    fn test_fmt() {
        let attr: Attribute = parse_quote!(#[fmt(|x| format!("0x{:x}", x))]);
        let attrs = ArgAttributes::from_attrs(&[attr]).unwrap();
        let value_: syn::Ident = parse_quote!(value);
        let result = attrs.generate_format_tokens(&value_);
        let expected = quote::quote! {
            (|x| format!("0x{:x}", x))(&value)
        };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_no_fmt() {
        let attrs = ArgAttributes::from_attrs(&[]).unwrap();
        let value_: syn::Ident = parse_quote!(value);
        let result = attrs.generate_format_tokens(&value_);
        let expected: TokenStream = quote::quote! {
            format!("{:?}", value)
        };
        assert_eq!(result.to_string(), expected.to_string());
    }
}

use syn::{Attribute, Expr};

/// Represents different types of argument attributes
#[derive(Clone)]
pub enum ArgAttribute {
    /// #\[show\] - Include in debug output with default formatting
    Show,
    /// #\[show(expr)\] - Include in debug output with custom formatter
    ShowWithExpression { formatter: Expr },
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
            if attr.path().is_ident("show") {
                if let Ok(formatter) = attr.parse_args::<Expr>() {
                    parsed_attrs.push(ArgAttribute::ShowWithExpression { formatter });
                } else {
                    parsed_attrs.push(ArgAttribute::Show);
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
            if let ArgAttribute::ShowWithExpression { formatter } = attr {
                Some(formatter)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_show_with_expression() {
        let attr: Attribute = parse_quote!(#[show(format!("0x{:x}", value))]);
        let attrs = ArgAttributes::from_attrs(&[attr]);
        assert!(attrs.should_print());
        assert!(attrs.get_custom_formatter().is_some());
    }

    #[test]
    fn test_show_with_field_access() {
        let attr: Attribute = parse_quote!(#[show(node.key)]);
        let attrs = ArgAttributes::from_attrs(&[attr]);
        assert!(attrs.should_print());
        assert!(attrs.get_custom_formatter().is_some());
    }

    #[test]
    fn test_basic_show() {
        let attr: Attribute = parse_quote!(#[show]);
        let attrs = ArgAttributes::from_attrs(&[attr]);
        assert!(attrs.should_print());
        assert!(attrs.get_custom_formatter().is_none());
    }

    #[test]
    fn test_no_show() {
        let attrs = ArgAttributes::from_attrs(&[]);
        assert!(!attrs.should_print());
    }
}

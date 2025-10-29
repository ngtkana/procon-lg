use syn::Attribute;

/// Check if the `#[no_debug]` attribute is present
pub fn has_no_debug_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("no_debug"))
}

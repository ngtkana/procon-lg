use syn::{ReturnType, Type};

/// Check if the return type is unit type ()
pub fn is_unit_return_type(return_type: &ReturnType) -> bool {
    match return_type {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => {
            if let Type::Tuple(tuple) = &**ty {
                tuple.elems.is_empty()
            } else {
                false
            }
        }
    }
}

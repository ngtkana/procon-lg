//! # procon-lg
//!
//! A procedural macro library for debugging recursive functions in competitive programming

mod args;
mod arg_attrs;
mod codegen;
mod visitor;

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use args::MacroArgs;
use codegen::CodeGenerator;

/// A procedural macro for logging recursive function calls
///
/// # Usage
///
/// ```rust
/// use procon_lg::lg_recur;
///
/// #[lg_recur]
/// fn fibonacci(n: u32) -> u32 {
///     if n <= 1 {
///         1
///     } else {
///         fibonacci(n - 1) + fibonacci(n - 2)
///     }
/// }
/// ```
///
/// # Options
///
/// - `no_return`: Disable return value output
///
/// ```rust
/// #[lg_recur(no_return)]
/// fn some_function() -> SomeType {
///     // implementation
/// }
/// ```
///
/// # Attributes
///
/// - `#[no_debug]`: Exclude specific arguments from debug output
/// - `#[fmt(closure)]`: Use custom formatter for argument display
///
/// ```rust
/// #[lg_recur]
/// fn process(
///     #[no_debug] secret: Secret, 
///     #[fmt(|node: &Node| node.key)] node: &Node,
///     #[fmt(|x: &u32| format!("0x{:x}", x))] hex_value: u32,
/// ) {
///     // secret argument will not be included in debug output
///     // node will be displayed using the custom formatter (showing only the key)
///     // hex_value will be displayed in hexadecimal format
/// }
/// ```
#[proc_macro_attribute]
pub fn lg_recur(attr: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = if attr.is_empty() {
        MacroArgs::default()
    } else {
        parse_macro_input!(attr as MacroArgs)
    };
    let input_fn = parse_macro_input!(item as ItemFn);
    let code_generator = CodeGenerator {
        input_fn,
        macro_args,
    };
    let expanded = code_generator.generate();
    TokenStream::from(expanded)
}

//! # procon-lg
//!
//! A procedural macro library for debugging recursive functions in competitive programming

mod arg_attrs;
mod args;
mod codegen;
mod visitor;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

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
/// - `recursion_limit = N`: Set maximum recursion depth limit (must be > 0)
///
/// ```rust,no_run
/// use procon_lg::lg_recur;
///
/// struct SomeType;
///
/// #[lg_recur(no_return)]
/// fn some_function(x: i32) -> SomeType {
///     SomeType
/// }
///
/// #[lg_recur(recursion_limit = 100)]
/// fn fibonacci(n: u32) -> u32 {
///     if n <= 1 {
///         1
///     } else {
///         fibonacci(n - 1) + fibonacci(n - 2)
///     }
/// }
///
/// #[lg_recur(no_return, recursion_limit = 50)]
/// fn limited_function(x: i32) -> SomeType {
///     SomeType
/// }
/// ```
///
/// # Attributes
///
/// - `#[no_debug]`: Exclude specific arguments from debug output
/// - `#[fmt(closure)]`: Use custom formatter for argument display
/// - `#[no_name]`: Hide argument name (only show value, not "arg=value")
///
/// ```rust,no_run
/// use procon_lg::lg_recur;
///
/// struct Secret;
/// struct Node { key: i32 }
///
/// #[lg_recur]
/// fn process(
///     #[no_debug] secret: Secret,
///     #[fmt(node.key)] node: &Node,
///     #[fmt(format!("0x{:x}", hex_value))] #[no_name] hex_value: u32,
///     #[no_name] count: i32,
/// ) {
///     // secret argument will not be included in debug output
///     // node will be displayed using the custom formatter (showing only the key)
///     // hex_value will be displayed in hexadecimal format without "hex_value=" prefix
///     // count will be displayed without "count=" prefix
/// }
/// ```
///
/// Multiple attributes can be combined on the same argument.
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

//! # procon-lg
//!
//! A procedural macro library for debugging recursive functions in competitive programming

#![allow(unused_attributes)]
#![feature(proc_macro_hygiene)]

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
/// - `recursion_limit = N`: Set maximum recursion depth limit (must be > 0)
///
/// ```rust,no_run
/// use procon_lg::lg_recur;
///
/// #[lg_recur(recursion_limit = 100)]
/// fn fibonacci(n: u32) -> u32 {
///     if n <= 1 {
///         1
///     } else {
///         fibonacci(n - 1) + fibonacci(n - 2)
///     }
/// }
/// ```
///
/// # Attributes
///
/// By default, no arguments or return values are printed. Use these attributes to opt-in:
///
/// - `#[fmt]`: Include specific arguments in debug output with default formatting
/// - `#[fmt(expr)]`: Include specific arguments with custom formatter
///
/// ```rust,no_run
/// use procon_lg::lg_recur;
///
/// struct Node { key: i32 }
///
/// #[lg_recur]
/// fn process(
///     #[fmt] count: i32,
///     #[fmt(node.key)] node: &Node,
///     #[fmt(format!("0x{:x}", hex_value))] hex_value: u32,
///     hidden_arg: u32,  // This will not be printed
/// ) {
///     // Only count, node.key, and hex_value (in hex format) will be printed
///     // hidden_arg will not be included in debug output
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

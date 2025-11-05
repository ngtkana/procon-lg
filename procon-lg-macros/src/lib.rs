//! # procon-lg
//!
//! A procedural macro library for debugging recursive functions in competitive programming

mod arg_attrs;
mod codegen;
mod macro_args;
mod visitor;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

use codegen::CodeGenerator;
use macro_args::MacroArgs;

/// A procedural macro for logging recursive function calls
///
/// This is the implementation crate for the `lg_recur` macro.
/// For usage examples, see the main `procon-lg` crate documentation.
///
/// # Options
///
/// - `no_return`: Disable return value output
/// - `recursion_limit = N`: Set maximum recursion depth limit (must be > 0)
///
/// # Attributes
///
/// - `#[show]`: Include argument in debug output with default formatting
/// - `#[show(expr)]`: Include argument in debug output with custom formatter expression
#[proc_macro_attribute]
pub fn lg_recur(attr: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args: MacroArgs = if attr.is_empty() {
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

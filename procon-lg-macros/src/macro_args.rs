use syn::parse::Parse;

/// Structure representing macro arguments
#[derive(Default)]
pub struct MacroArgs {
    pub recursion_limit: Option<usize>,
    pub show_return: bool,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = MacroArgs::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;

            match ident.to_string().as_str() {
                "recursion_limit" => {
                    input.parse::<syn::Token![=]>()?;
                    let limit: syn::LitInt = input.parse()?;
                    let limit_value = limit.base10_parse::<usize>()?;
                    if limit_value == 0 {
                        return Err(syn::Error::new(
                            limit.span(),
                            "recursion_limit must be greater than 0",
                        ));
                    }
                    args.recursion_limit = Some(limit_value);
                }
                "show_return" => {
                    args.show_return = true;
                }
                _ => return Err(syn::Error::new(ident.span(), "unknown argument")),
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(args)
    }
}

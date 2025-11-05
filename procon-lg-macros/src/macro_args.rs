use syn::parse::Parse;

/// Structure representing macro arguments
#[derive(Default)]
pub struct MacroArgs {
    pub no_return: bool,
    pub recursion_limit: Option<usize>,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = MacroArgs::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;

            match ident.to_string().as_str() {
                "no_return" => args.no_return = true,
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
                _ => return Err(syn::Error::new(ident.span(), "unknown argument")),
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(args)
    }
}

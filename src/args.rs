use syn::parse::Parse;

/// Structure representing macro arguments
#[derive(Default)]
pub struct MacroArgs {
    pub no_return: bool,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = MacroArgs::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;

            match ident.to_string().as_str() {
                "no_return" => args.no_return = true,
                _ => return Err(syn::Error::new(ident.span(), "unknown argument")),
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(args)
    }
}

impl MacroArgs {
    /// Create default arguments
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the `no_return` flag is set
    pub fn has_no_return(&self) -> bool {
        self.no_return
    }
}

pub struct Texter {
    key: syn::Ident,
    args: Vec<syn::Expr>
}
impl syn::parse::Parse for Texter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse pattern:
        // IDENT (, EXPR)*

        // Gets IDENT
        let key = input.parse()?;

        // Gets any number of (, EXPR)
        let mut args = Vec::new();
        while let Ok(arg) = parse_arg(input) {
            args.push(arg);
        }
        
        Ok(Self {
            key,
            args,
        })
    }
}
impl quote::ToTokens for Texter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { key, args } = &self;

        let new = quote::quote! {
            localisation::#key(
                localisation::get_locale() 
                #(,#args)*
            )
        };

        tokens.extend(new);
    }
}

fn parse_arg(input: syn::parse::ParseStream) -> syn::Result<syn::Expr> {
    _ = input.parse::<syn::Token![,]>()?;
    input.parse()
}

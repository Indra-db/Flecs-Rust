use syn::{
    Ident, LitInt, Result,
    parse::{Parse, ParseStream},
    token::Comma,
};

pub struct Tuples {
    pub macro_ident: Ident,
    pub start: usize,
    pub end: usize,
    pub idents: Vec<Ident>,
}

impl Parse for Tuples {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let start = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let end = input.parse::<LitInt>()?.base10_parse()?;
        let mut idents: Vec<Ident> = Vec::new();
        while input.parse::<Comma>().is_ok() {
            let ident = input.parse::<Ident>()?;
            idents.push(ident);
        }

        Ok(Tuples {
            macro_ident,
            start,
            end,
            idents,
        })
    }
}

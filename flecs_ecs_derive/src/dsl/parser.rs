// DSL parser and builder structures

use proc_macro2::TokenStream;
use syn::{
    Expr, LitStr, Result, Token, Type,
    parse::{Parse, ParseStream},
};
use quote::quote;

use super::term::Term;

/// Complete DSL with all terms
pub struct Dsl {
    pub terms: Vec<Term>,
    //TODO 2024 edition doesn't support it anymore. Need to find workaround
    pub _doc: Option<TokenStream>,
}

impl Parse for Dsl {
    fn parse(input: ParseStream) -> Result<Self> {
        let string = input.cursor().token_stream().to_string();
        let stripped = string
            .replace('\"', "")
            .replace("& mut", "")
            .replace('&', "")
            .replace(" ,", ",");
        let string = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
        let doc = syn::parse_str::<TokenStream>(&format!("#[doc = \"{string}\"]")).ok();
        let doc = doc.map(|doc| {
            quote! {
                #[allow(clippy::suspicious_doc_comments)]
                #doc
                const _: () = ();
            }
        });

        let mut terms = Vec::new();
        terms.push(input.parse::<Term>()?);
        while input.peek(Token![,]) || input.peek(Token![|]) {
            if input.peek(Token![|]) {
                input.parse::<Token![|]>()?;
                input.parse::<Token![|]>()?;
                terms.last_mut().unwrap().oper = super::types::TermOper::Or;
            } else {
                input.parse::<Token![,]>()?;

                // Handle optional trailing comma
                if input.is_empty() {
                    break;
                }
            }
            terms.push(input.parse::<Term>()?);
        }

        Ok(Dsl { terms, _doc: doc })
    }
}

/// Builder structure for queries and systems
pub struct Builder {
    pub name: Option<LitStr>,
    pub world: Expr,
    pub dsl: Dsl,
}

impl Parse for Builder {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = if input.peek(LitStr) {
            let name = input.parse::<LitStr>()?;
            input.parse::<Token![,]>()?;
            Some(name)
        } else {
            None
        };
        let world = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let dsl = input.parse::<Dsl>()?;

        Ok(Builder { name, world, dsl })
    }
}

/// Observer structure
pub struct Observer {
    pub name: Option<LitStr>,
    pub world: Expr,
    pub event: Type,
    pub dsl: Dsl,
}

impl Parse for Observer {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = if input.peek(LitStr) {
            let name = input.parse::<LitStr>()?;
            input.parse::<Token![,]>()?;
            Some(name)
        } else {
            None
        };
        let world = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let event = input.parse::<Type>()?;
        input.parse::<Token![,]>()?;
        let dsl = input.parse::<Dsl>()?;

        Ok(Observer {
            name,
            world,
            event,
            dsl,
        })
    }
}

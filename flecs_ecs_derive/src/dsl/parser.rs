// DSL parser and builder structures

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, LitStr, Result, Token, Type,
    parse::{Parse, ParseStream},
};

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
        
        // Parse the first term (which might be a parenthesized group with OR expressions)
        let first_terms = parse_term_or_group(input)?;
        terms.extend(first_terms);
        
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
            let next_terms = parse_term_or_group(input)?;
            terms.extend(next_terms);
        }

        Ok(Dsl { terms, _doc: doc })
    }
}

/// Parse a single term or a parenthesized group containing multiple terms with OR
fn parse_term_or_group(input: ParseStream) -> Result<Vec<Term>> {
    // Check if this is a parenthesized equality expression with OR
    if input.peek(syn::token::Paren) {
        let lookahead = input.fork();
        let inner;
        syn::parenthesized!(inner in lookahead);
        
        // Check if it looks like an equality expression by looking for $variable
        if inner.peek(Token![$]) {
            // Look ahead to detect OR operator or equality operators
            // We need to properly scan the token stream instead of converting to string
            let has_or = contains_or_operator(&inner);
            let has_equality = contains_equality_operator(&inner);            if has_or {
                // This is a group of OR'd equality expressions
                // Parse them as separate terms
                let paren_content;
                syn::parenthesized!(paren_content in input);
                
                let mut group_terms = Vec::new();
                group_terms.push(paren_content.parse::<Term>()?);
                
                while paren_content.peek(Token![|]) && paren_content.peek2(Token![|]) {
                    paren_content.parse::<Token![|]>()?;
                    paren_content.parse::<Token![|]>()?;
                    // Check if the previous term already has a Not operator
                    // If so, combine it with Or to create NotOr
                    let last_term = group_terms.last_mut().unwrap();
                    if last_term.oper == super::types::TermOper::Not {
                        last_term.oper = super::types::TermOper::NotOr;
                    } else {
                        last_term.oper = super::types::TermOper::Or;
                    }
                    group_terms.push(paren_content.parse::<Term>()?);
                }
                
                return Ok(group_terms);
            } else if has_equality {
                // This is a single parenthesized equality expression
                let paren_content;
                syn::parenthesized!(paren_content in input);
                return Ok(vec![paren_content.parse::<Term>()?]);
            }
        }
    }
    
    // Not a special case, parse as normal term
    Ok(vec![input.parse::<Term>()?])
}

/// Check if a ParseStream contains the OR operator (||)
fn contains_or_operator(input: ParseStream) -> bool {
    let mut cursor = input.cursor();
    while let Some((tt, next)) = cursor.token_tree() {
        if let proc_macro2::TokenTree::Punct(punct) = tt {
            if punct.as_char() == '|' {
                // Check if the next token is also |
                if let Some((proc_macro2::TokenTree::Punct(next_punct), _)) = next.token_tree() {
                    if next_punct.as_char() == '|' {
                        return true;
                    }
                }
            }
        }
        cursor = next;
    }
    false
}

/// Check if a ParseStream contains an equality operator (==, !=, ~=)
fn contains_equality_operator(input: ParseStream) -> bool {
    let mut cursor = input.cursor();
    while let Some((tt, next)) = cursor.token_tree() {
        if let proc_macro2::TokenTree::Punct(punct) = tt {
            match punct.as_char() {
                '=' | '!' | '~' => {
                    // Check if the next token is =
                    if let Some((proc_macro2::TokenTree::Punct(next_punct), _)) = next.token_tree() {
                        if next_punct.as_char() == '=' {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        cursor = next;
    }
    false
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

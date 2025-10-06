// Core types for DSL parsing

use proc_macro2::TokenStream;
use syn::{
    Ident, LitStr, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    token::Bracket, bracketed,
};

/// Reference type for terms (&, &mut, or none)
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Reference {
    Mut,
    Ref,
    #[default]
    None,
}

impl Parse for Reference {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![&]) {
            input.parse::<Token![&]>()?;
            if input.peek(Token![mut]) {
                input.parse::<Token![mut]>()?;
                Ok(Reference::Mut)
            } else {
                Ok(Reference::Ref)
            }
        } else {
            Ok(Reference::None)
        }
    }
}

/// Access specifier for terms ([in], [out], [inout], [filter], [none])
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Access {
    In,
    Out,
    InOut,
    Filter,
    None,
    #[default]
    Omitted,
}

impl Parse for Access {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Bracket) {
            let inner;
            bracketed!(inner in input);
            if inner.peek(Token![in]) {
                inner.parse::<Token![in]>()?;
                Ok(Access::In)
            } else if inner.peek(kw::out) {
                inner.parse::<kw::out>()?;
                Ok(Access::Out)
            } else if inner.peek(kw::inout) {
                inner.parse::<kw::inout>()?;
                Ok(Access::InOut)
            } else if inner.peek(kw::filter) {
                inner.parse::<kw::filter>()?;
                Ok(Access::Filter)
            } else if inner.peek(kw::none) {
                inner.parse::<kw::none>()?;
                Ok(Access::None)
            } else {
                Ok(Access::Omitted)
            }
        } else {
            Ok(Access::Omitted)
        }
    }
}

/// Identifier type for terms (can be a type, variable, literal, etc.)
pub enum TermIdent {
    Local(Ident),
    Variable(LitStr),
    Type(Type),
    EnumType(Path),
    Literal(LitStr),
    SelfType,
    SelfVar,
    Wildcard,
    Any,
}

impl Parse for TermIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Ok(TermIdent::Wildcard)
        } else if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            Ok(TermIdent::Any)
        } else if input.peek(Token![$]) {
            // Variable
            input.parse::<Token![$]>()?;
            if input.peek(Ident) {
                Ok(TermIdent::Local(input.parse::<Ident>()?))
            } else if input.peek(LitStr) {
                Ok(TermIdent::Variable(input.parse::<LitStr>()?))
            } else if input.peek(Token![self]) {
                input.parse::<Token![self]>()?;
                Ok(TermIdent::SelfVar)
            } else {
                panic!(
                    "unexpected token after `self`, token: {:?}",
                    input.cursor().token_stream()
                );
            }
        } else if input.peek(LitStr) {
            Ok(TermIdent::Literal(input.parse::<LitStr>()?))
        } else if input.peek(Token![Self]) {
            input.parse::<Token![Self]>()?;
            Ok(TermIdent::SelfType)
        } else if input.peek(kw::variant) {
            input.parse::<kw::variant>()?;
            Ok(TermIdent::EnumType(input.parse::<Path>()?))
        } else {
            Ok(TermIdent::Type(input.parse::<Type>()?))
        }
    }
}

/// Helper to check if the next token could be an identifier
pub(crate) fn peek_id(input: &ParseStream) -> bool {
    input.peek(Ident)
        || input.peek(Token![*])
        || input.peek(Token![_])
        || input.peek(Token![$])
        || input.peek(LitStr)
        || input.peek(Token![Self])
}

/// Operator type for terms (not, optional, and|, not|, or|, or, and)
#[derive(Default, Debug, PartialEq, Eq)]
pub enum TermOper {
    Not,
    Optional,
    AndFrom,
    NotFrom,
    OrFrom,
    Or,
    #[default]
    And,
}

/// Custom keywords used in the DSL
pub mod kw {
    // Operators
    syn::custom_keyword!(and);
    syn::custom_keyword!(not);
    syn::custom_keyword!(or);

    // Traversal
    syn::custom_keyword!(cascade);
    syn::custom_keyword!(desc);
    syn::custom_keyword!(up);

    // Access
    syn::custom_keyword!(out);
    syn::custom_keyword!(inout);
    syn::custom_keyword!(filter);
    syn::custom_keyword!(none);

    // For flecs enum type
    syn::custom_keyword!(variant);
}

impl Parse for TermOper {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::and) {
            input.parse::<kw::and>()?;
            input.parse::<Token![|]>()?;
            Ok(TermOper::AndFrom)
        } else if input.peek(kw::not) {
            input.parse::<kw::not>()?;
            input.parse::<Token![|]>()?;
            Ok(TermOper::NotFrom)
        } else if input.peek(kw::or) {
            input.parse::<kw::or>()?;
            input.parse::<Token![|]>()?;
            Ok(TermOper::OrFrom)
        } else if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            Ok(TermOper::Not)
        } else if input.peek(Token![?]) {
            input.parse::<Token![?]>()?;
            Ok(TermOper::Optional)
        } else {
            Ok(TermOper::And)
        }
    }
}

/// Helper to check if the next token is a traversal keyword
pub(crate) fn peek_trav(input: ParseStream) -> bool {
    input.peek(kw::cascade)
        || input.peek(kw::desc)
        || input.peek(kw::up)
        || input.peek(Token![self])
}

/// Expands a TermIdent to its TokenStream representation for types
pub fn expand_type(ident: &TermIdent) -> Option<TokenStream> {
    use quote::quote;
    
    match ident {
        TermIdent::Type(ty) => Some(quote! { #ty }),
        TermIdent::EnumType(ty) => Some(quote! { #ty }),
        TermIdent::Wildcard => Some(quote! { flecs_ecs::core::flecs::Wildcard }),
        TermIdent::Any => Some(quote! { flecs_ecs::core::flecs::Any }),
        TermIdent::SelfType => Some(quote! { Self }),
        _ => None,
    }
}

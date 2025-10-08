//! Core types for DSL parsing
//!
//! This module defines the fundamental types used throughout the DSL implementation:
//! - `Reference`: Tracks Rust reference types (&, &mut, none)
//! - `Access`: Tracks Flecs access specifiers ([in], [out], etc.)
//! - `TermIdent`: Different kinds of component identifiers
//! - `TermOper`: Query operators (!, ?, ||, etc.)
//!
//! # Examples
//!
//! ```ignore
//! // Reference types
//! &Position        → Reference::Ref
//! &mut Velocity    → Reference::Mut
//! Tag              → Reference::None
//!
//! // Access specifiers
//! [in] Position    → Access::In
//! [out] Velocity   → Access::Out
//! [filter] Tag     → Access::Filter
//!
//! // Term identifiers
//! Position         → TermIdent::Type
//! "Position"       → TermIdent::Literal
//! $var             → TermIdent::Variable
//! *                → TermIdent::Wildcard
//! ```

use proc_macro2::TokenStream;
use syn::{
    Ident, LitStr, Path, Result, Token, Type, bracketed,
    parse::{Parse, ParseStream},
    token::Bracket,
};

/// Reference type for terms (&, &mut, or none)
///
/// Determines how a component is accessed in the query iterator:
/// - `Ref`: Immutable reference, allows multiple readers
/// - `Mut`: Mutable reference, exclusive access
/// - `None`: No reference, component is filtered but not accessed
///
/// # Examples
///
/// ```ignore
/// query!(world, &Position)      // Reference::Ref
/// query!(world, &mut Velocity)  // Reference::Mut  
/// query!(world, Tag)            // Reference::None (filter only)
/// ```
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Reference {
    /// Mutable reference (&mut T)
    Mut,
    /// Immutable reference (&T)
    Ref,
    /// No reference - component exists but is not accessed
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
///
/// Controls how Flecs handles component access:
/// - `In`: Read-only access (optimization hint)
/// - `Out`: Write-only access, current value not read
/// - `InOut`: Read-write access
/// - `Filter`: Component matched but not accessed (most efficient)
/// - `None`: No data access needed
/// - `Omitted`: No explicit access specified (inferred from reference type)
///
/// # Examples
///
/// ```ignore
/// query!(world, [in] Position)      // Access::In
/// query!(world, [out] &mut Vel)     // Access::Out
/// query!(world, [filter] Active)    // Access::Filter
/// query!(world, &Position)          // Access::Omitted (inferred as In)
/// ```
///
/// # Performance
///
/// - `Filter`: Most efficient, no data access
/// - `In`: Allows parallel access
/// - `Out`: Optimization when current value not needed
/// - `InOut`: Full access, may block parallel queries
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Access {
    /// Read-only access [in]
    In,
    /// Write-only access [out]
    Out,
    /// Read-write access [inout]
    InOut,
    /// Match only, no access [filter]
    Filter,
    /// No access needed [none]
    None,
    /// No explicit access specifier (inferred)
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
///
/// Represents different ways to identify components in the DSL:
///
/// # Variants
///
/// - `Local`: Local Rust identifier (e.g., `$my_id`)
/// - `Variable`: Named query variable (e.g., `$"parent"`)
/// - `Type`: Rust type (e.g., `Position`)
/// - `EnumType`: Flecs enum type (e.g., `variant Color`)
/// - `Literal`: String literal name (e.g., `"Position"`)
/// - `SelfType`: The `Self` type in generic contexts
/// - `SelfVar`: The `$self` variable for current entity
/// - `Wildcard`: Wildcard matcher `*` (matches any)
/// - `Any`: Universal matcher `_` (matches anything)
///
/// # Examples
///
/// ```ignore
/// query!(world, Position)           // TermIdent::Type
/// query!(world, "Position")         // TermIdent::Literal
/// query!(world, $parent)            // TermIdent::Local
/// query!(world, $"parent")          // TermIdent::Variable
/// query!(world, *)                  // TermIdent::Wildcard
/// query!(world, _)                  // TermIdent::Any
/// query!(world, variant Color)      // TermIdent::EnumType
/// query!(world, Self)               // TermIdent::SelfType
/// query!(world, $self)              // TermIdent::SelfVar
/// ```
pub enum TermIdent {
    /// Local Rust identifier: `$my_id`
    Local(Ident),
    /// Named query variable: `$"parent"`
    Variable(LitStr),
    /// Rust type: `Position`
    Type(Type),
    /// Flecs enum type: `variant Color`
    EnumType(Path),
    /// String literal: `"Position"`
    Literal(LitStr),
    /// Self type in generic context
    SelfType,
    /// Self variable for current entity
    SelfVar,
    /// Wildcard: `*` (matches any)
    Wildcard,
    /// Universal: `_` (matches anything)
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
///
/// Controls how terms are combined in the query:
///
/// # Operators
///
/// - `And`: Default, all terms must match (implicit)
/// - `Or`: Either this term or the next must match (`||`)
/// - `Not`: Term must not match (`!`)
/// - `Optional`: Term may or may not match (`?`)
/// - `AndFrom`: Match with component from previous term (`and|`)
/// - `NotFrom`: Exclude component from previous term (`not|`)
/// - `OrFrom`: Match one of components from previous term (`or|`)
///
/// # Examples
///
/// ```ignore
/// query!(world, &Position, &Velocity)    // And (implicit)
/// query!(world, &Position, !Dead)        // Not
/// query!(world, &Position, ?&Velocity)   // Optional
/// query!(world, &Pos || &Vel)            // Or
/// query!(world, &Position, and| Parent)  // AndFrom
/// query!(world, &Position, not| Dead)    // NotFrom
/// query!(world, &Position, or| Source)   // OrFrom
/// ```
///
/// # Semantics
///
/// - `And`: A ∧ B (both must be true)
/// - `Or`: A ∨ B (at least one must be true)
/// - `Not`: A ∧ ¬B (A must be true, B must be false)
/// - `Optional`: A ∧ (B ∨ ¬B) (A must be true, B doesn't matter)
#[derive(Default, Debug, PartialEq, Eq)]
pub enum TermOper {
    /// Exclude entities with this term: `!Tag`
    Not,
    /// Term is optional: `?&Component`
    Optional,
    /// Match with component from previous term: `and| Source`
    AndFrom,
    /// Exclude component from previous term: `not| Dead`
    NotFrom,
    /// Match one from previous term: `or| Source`
    OrFrom,
    /// Either this or next term: `A || B`
    Or,
    /// Combination of Not and Or: `!A || B` (internal)
    NotOr,
    /// Both terms must match (implicit)
    #[default]
    And,
}

/// Equality operator type for comparison expressions
///
/// Used in advanced query expressions to compare variables with entities or names.
///
/// # Operators
///
/// - `Equal`: Matches exact entity or name (`==`)
/// - `NotEqual`: Negates exact match (`!=`)
/// - `Match`: Fuzzy string matching (`~=`)
///
/// # Examples
///
/// ```ignore
/// $this == UssEnterprise        // EqualityOper::Equal
/// $this != UssEnterprise        // EqualityOper::NotEqual
/// $this ~= "Uss"                // EqualityOper::Match
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EqualityOper {
    /// Exact match: `==`
    Equal,
    /// Not equal: `!=`
    NotEqual,
    /// Fuzzy match: `~=`
    Match,
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

/// Expands a `TermIdent` to its `TokenStream` representation for types
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

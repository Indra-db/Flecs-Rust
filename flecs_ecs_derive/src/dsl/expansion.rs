// Common expansion utilities for DSL macros
//
// This module provides low-level expansion functions used by the query, system,
// and observer macros. Most of the high-level builder logic has been moved to
// the `builder` module for better organization.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use super::term::{Term, TermId, TermType};
use super::types::{Reference, TermIdent, TermOper, expand_type};

/// Expands traversal operations for a TermId
///
/// Traversal operations control how queries traverse relationships in the ECS hierarchy.
/// This includes operations like `up`, `cascade`, `desc`, and `self`.
///
/// # Arguments
///
/// * `term` - The TermId containing traversal configuration
///
/// # Returns
///
/// A vector of TokenStreams representing the traversal method calls
pub fn expand_trav(term: &TermId) -> Vec<TokenStream> {
    let mut ops = Vec::new();
    if term.trav_up {
        match &term.up_ident {
            Some(ident) => match ident {
                TermIdent::Local(ident) => ops.push(quote! { .up_id(#ident) }),
                TermIdent::Type(ty) => ops.push(quote! { .up_id(id::<#ty>()) }),
                _ => ops
                    .push(quote_spanned!(term.span => ; compile_error!("Invalid up traversal.") )),
            },
            None => ops.push(quote! { .up() }),
        }
    }
    if term.trav_cascade {
        match &term.cascade_ident {
            Some(ident) => match ident {
                TermIdent::Local(ident) => ops.push(quote! { .cascade_id(#ident) }),
                TermIdent::Type(ty) => ops.push(quote! { .cascade_id(id::<#ty>()) }),
                _ => ops.push(
                    quote_spanned!(term.span => ; compile_error!("Invalid cascade traversal.") ),
                ),
            },
            None => ops.push(quote! { .cascade() }),
        }
    }
    if term.trav_desc {
        ops.push(quote! { .desc() });
    }
    if term.trav_self {
        ops.push(quote! { .self_() });
    }
    ops
}

/// Expands a term type to its iterator type representation
///
/// This function determines if a term should be part of the iterator type.
/// Terms with references (&, &mut) become part of the iterator, while
/// filter terms and terms without references do not.
///
/// # Arguments
///
/// * `term` - The term to expand
///
/// # Returns
///
/// An optional TokenStream representing the iterator type for this term
pub fn expand_term_type(term: &Term) -> Option<TokenStream> {
    let ty = match &term.ty {
        TermType::Pair(first, second) => {
            let first = first.ident.as_ref()?;
            let second = second.ident.as_ref()?;
            let first = expand_type(first)?;
            let second = expand_type(second)?;
            quote! { (#first, #second) }
        }
        TermType::Component(id) => {
            let id = id.ident.as_ref()?;
            expand_type(id)?
        }
        TermType::Equality(_) => {
            // Equality expressions are not part of the iterator type
            return None;
        }
    };

    let access_type = match term.reference {
        Reference::Mut => quote! { &mut #ty },
        Reference::Ref => quote! { & #ty },
        Reference::None => return None,
    };

    match &term.oper {
        TermOper::Optional => Some(quote! { Option<#access_type> }),
        TermOper::And => Some(quote! { #access_type }),
        _ => None,
    }
}

/// Expands DSL terms into iterator type and builder calls
///
/// This is a convenience re-export that delegates to the builder module.
/// This function is kept for backwards compatibility.
///
/// # Arguments
///
/// * `terms` - Mutable slice of terms to expand
///
/// # Returns
///
/// A tuple containing:
/// - The iterator type as a TokenStream
/// - A vector of builder call TokenStreams
pub fn expand_dsl(terms: &mut [Term]) -> (TokenStream, Vec<TokenStream>) {
    super::builder::build_query_components(terms)
}

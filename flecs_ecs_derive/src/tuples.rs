//! Internal utilities for generating tuple implementations.
//!
//! This module provides the `Tuples` struct for parsing tuple macro input
//! and the `expand_tuples` function for generating macro invocations.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
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

/// Expansion function for the `tuples` macro.
///
/// This generates macro invocations for a range of tuple sizes, allowing the library
/// to generate trait implementations for tuples of different arities.
///
/// # Arguments
///
/// * `input` - A `Tuples` struct containing the macro to invoke and the range of tuple sizes
///
/// # Returns
///
/// A `TokenStream` containing the generated macro invocations
pub(crate) fn expand_tuples(input: Tuples) -> TokenStream {
    let len = 1 + input.end - input.start;
    let mut tuples = Vec::with_capacity(len);
    for i in 0..=len {
        tuples.push(format_ident!("P{}", i));
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let tuples = &tuples[..i];
        let idents = &input.idents;

        quote! {
            #macro_ident!(#(#idents,)* #(#tuples),*);
        }
    });

    quote! {
        #(
            #invocations
        )*
    }
}

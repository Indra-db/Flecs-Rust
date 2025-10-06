// Query macro expansion

use proc_macro2::TokenStream;
use quote::quote;

use super::parser::Builder;
use super::expansion::expand_dsl;

/// Expansion function for the `query` macro.
///
/// Generates a query builder with the appropriate method calls based on the DSL terms.
///
/// # Arguments
///
/// * `input` - A `Builder` struct containing the query name, world, and DSL terms
///
/// # Returns
///
/// A `TokenStream` containing the generated query builder code
pub fn expand_query(input: Builder) -> TokenStream {
    let mut terms = input.dsl.terms;
    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let world = input.world;

    match input.name {
        Some(name) => quote! {
            (#world).query_named::<#iter_type>(#name)
            #(
                #builder_calls
            )*
        },
        None => quote! {
            (#world).query::<#iter_type>()
            #(
                #builder_calls
            )*
        },
    }
}

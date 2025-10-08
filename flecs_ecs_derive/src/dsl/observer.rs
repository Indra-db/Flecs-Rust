// Observer macro expansion

use proc_macro2::TokenStream;
use quote::quote;

use super::expansion::expand_dsl;
use super::parser::Observer;

/// Expansion function for the `observer` macro.
///
/// Generates an observer builder with the appropriate method calls based on the DSL terms.
///
/// # Arguments
///
/// * `input` - An `Observer` struct containing the observer name, world, event type, and DSL terms
///
/// # Returns
///
/// A `TokenStream` containing the generated observer builder code
pub fn expand_observer(input: Observer) -> TokenStream {
    let mut terms = input.dsl.terms;
    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let event_type = input.event;
    let world = input.world;

    match input.name {
        Some(name) => quote! {
            (#world).observer_named::<#event_type, #iter_type>(#name)
            #(
                #builder_calls
            )*
        },
        None => quote! {
            (#world).observer::<#event_type, #iter_type>()
            #(
                #builder_calls
            )*
        },
    }
}

// Helper functions for expanding TermIdent into builder calls

use proc_macro2::TokenStream;
use quote::quote;

use super::types::TermIdent;

/// Expands a TermIdent for use in a first/second position of a pair
pub fn expand_pair_component(
    ident: &TermIdent,
    iter_term: bool,
    position: PairPosition,
) -> Vec<TokenStream> {
    let mut ops = Vec::new();

    match ident {
        TermIdent::Variable(var) => {
            let var_name = format!("${}", var.value());
            let setter = match position {
                PairPosition::First => quote! { .set_first(#var_name) },
                PairPosition::Second => quote! { .set_second(#var_name) },
            };
            ops.push(setter);
        }
        TermIdent::SelfVar => {
            let setter = match position {
                PairPosition::First => quote! { .set_first(self) },
                PairPosition::Second => quote! { .set_second(self) },
            };
            ops.push(setter);
        }
        TermIdent::Local(ident) => {
            let setter = match position {
                PairPosition::First => quote! { .set_first(#ident) },
                PairPosition::Second => quote! { .set_second(#ident) },
            };
            ops.push(setter);
        }
        TermIdent::Literal(lit) => {
            let setter = match position {
                PairPosition::First => quote! { .set_first(#lit) },
                PairPosition::Second => quote! { .set_second(#lit) },
            };
            ops.push(setter);
        }
        _ => {
            if !iter_term {
                if let Some(ty) = super::types::expand_type(ident) {
                    let setter = match position {
                        PairPosition::First => quote! { .set_first(id::<#ty>()) },
                        PairPosition::Second => quote! { .set_second(id::<#ty>()) },
                    };
                    ops.push(setter);
                }
            }
        }
    }

    ops
}

/// Expands a TermIdent for use as a source
pub fn expand_source(ident: &TermIdent) -> TokenStream {
    match ident {
        TermIdent::Variable(var) => {
            let var_name = format!("${}", var.value());
            quote! { .set_src(#var_name) }
        }
        TermIdent::SelfVar => quote! { .set_src(self) },
        TermIdent::Local(ident) => quote! { .set_src(#ident) },
        TermIdent::Literal(lit) => quote! { .set_src(#lit) },
        _ => {
            if let Some(ty) = super::types::expand_type(ident) {
                quote! { .set_src(id::<#ty>()) }
            } else {
                quote! {}
            }
        }
    }
}

/// Position of a component in a pair
pub enum PairPosition {
    First,
    Second,
}

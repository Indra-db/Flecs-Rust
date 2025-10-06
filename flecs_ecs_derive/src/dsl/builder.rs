// Builder call generation for DSL terms

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use super::term::{Term, TermType};
use super::types::{Access, Reference, TermIdent, TermOper, expand_type};
use super::expansion::{expand_trav, expand_term_type};
use super::ident_expander::{expand_pair_component, expand_source, PairPosition};

/// Generate builder calls for a pair term
fn expand_pair_builder_calls(
    first: &super::term::TermId,
    second: &super::term::TermId,
    iter_term: bool,
    ops: &mut Vec<TokenStream>,
) {
    let first_id = first.ident.as_ref().expect("Pair with no first.");
    let second_id = second.ident.as_ref().expect("Pair with no second.");

    // Expand first component
    ops.extend(expand_pair_component(first_id, iter_term, PairPosition::First));
    
    // Expand second component
    ops.extend(expand_pair_component(second_id, iter_term, PairPosition::Second));

    // Configure traversal for first
    let id_ops = expand_trav(first);
    if !id_ops.is_empty() {
        ops.push(quote! { .first() #( #id_ops )* });
    }

    // Configure traversal for second
    let id_ops = expand_trav(second);
    if !id_ops.is_empty() {
        ops.push(quote! { .second() #( #id_ops )* });
    }
}

/// Generate builder calls for a component term
fn expand_component_builder_calls(
    term: &super::term::TermId,
    iter_term: bool,
    term_accessor: &mut TokenStream,
    needs_accessor: &mut bool,
    ops: &mut Vec<TokenStream>,
) {
    let id = term.ident.as_ref().expect("Term with no component.");
    let ty = expand_type(id);

    match id {
        TermIdent::Variable(var) => {
            let var_name = var.value();
            ops.push(quote! { .set_var(#var_name) });
        }
        TermIdent::SelfVar => ops.push(quote! { .set_id(self) }),
        TermIdent::Local(ident) => ops.push(quote! { .set_id(#ident) }),
        TermIdent::Literal(lit) => ops.push(quote! { .name(#lit) }),
        TermIdent::EnumType(_) => {
            if !iter_term {
                *term_accessor = quote! { .with_enum(#ty) };
                *needs_accessor = true;
            }
        }
        _ => {
            if !iter_term {
                *term_accessor = quote! { .with(id::<#ty>()) };
                *needs_accessor = true;
            }
        }
    };

    // Configure traversal
    let id_ops = expand_trav(term);
    if !id_ops.is_empty() {
        ops.push(quote! { #( #id_ops )* });
    }
}

/// Generate builder calls for operator configuration
fn expand_operator_builder_calls(
    oper: &TermOper,
    iter_term: bool,
    span: proc_macro2::Span,
    ops: &mut Vec<TokenStream>,
) {
    if iter_term {
        if !matches!(oper, TermOper::And | TermOper::Optional) {
            ops.push(quote_spanned! {
                span => ; compile_error!("Only 'optional' and 'and' operators allowed for static terms.")
            });
        }
    } else {
        match oper {
            TermOper::Not => ops.push(quote! { .not() }),
            TermOper::Or => ops.push(quote! { .or() }),
            TermOper::AndFrom => ops.push(quote! { .and_from() }),
            TermOper::NotFrom => ops.push(quote! { .not_from() }),
            TermOper::OrFrom => ops.push(quote! { .or_from() }),
            TermOper::Optional => ops.push(quote! { .optional() }),
            TermOper::And => {}
        }
    }
}

/// Generate builder calls for access configuration
fn expand_access_builder_calls(
    access: Access,
    reference: Reference,
    iter_term: bool,
    span: proc_macro2::Span,
    ops: &mut Vec<TokenStream>,
) {
    if iter_term {
        if !matches!(access, Access::Omitted | Access::Filter) {
            ops.push(quote_spanned! {
                span => ; compile_error!("Only [filter] is allowed on static terms.")
            });
        }

        if access == Access::Filter {
            ops.push(quote! { .filter() });
        }
    } else {
        match reference {
            Reference::None => {}
            _ => ops.push(quote_spanned! {
                span => ; compile_error!("Static term located after a dynamic term, re-order such that `&` and `&mut` are first.")
            }),
        }

        match access {
            Access::In => ops.push(quote! { .set_in() }),
            Access::Out => ops.push(quote! { .set_out() }),
            Access::InOut => ops.push(quote! { .set_inout() }),
            Access::Filter => ops.push(quote! { .filter() }),
            Access::None => ops.push(quote! { .set_inout_none() }),
            Access::Omitted => {}
        }
    }
}

/// Expands a single term into builder calls
pub fn expand_term_builder_calls(
    term: &Term,
    index: u32,
    iter_term: bool,
) -> Option<TokenStream> {
    let mut ops = Vec::new();
    let mut needs_accessor = false;
    let mut term_accessor = if !iter_term {
        quote! { .term() }
    } else {
        quote! { .term_at(#index) }
    };

    // Expand term type (component or pair)
    match &term.ty {
        TermType::Pair(first, second) => {
            expand_pair_builder_calls(first, second, iter_term, &mut ops);
        }
        TermType::Component(component) => {
            expand_component_builder_calls(
                component,
                iter_term,
                &mut term_accessor,
                &mut needs_accessor,
                &mut ops,
            );
        }
    }

    // Configure source
    if let Some(source) = &term.source.ident {
        ops.push(expand_source(source));
    }

    // Configure operator
    expand_operator_builder_calls(&term.oper, iter_term, term.span, &mut ops);

    // Configure traversal for source
    let id_ops = expand_trav(&term.source);
    if !id_ops.is_empty() {
        ops.push(quote! { .src() #( #id_ops )* });
    }

    // Configure access
    expand_access_builder_calls(term.access, term.reference, iter_term, term.span, &mut ops);

    if !ops.is_empty() || needs_accessor {
        Some(quote! {
            #term_accessor
            #( #ops )*
        })
    } else {
        None
    }
}

/// Generates the iterator type and builder calls for a list of terms
pub fn build_query_components(terms: &mut [Term]) -> (TokenStream, Vec<TokenStream>) {
    // Collect iterator terms (terms with references that become part of the iterator type)
    let mut iter_terms = Vec::new();
    for t in terms.iter() {
        match expand_term_type(t) {
            Some(ty) => iter_terms.push(ty),
            None => break,
        }
    }

    // Generate iterator type
    let iter_type = if iter_terms.len() == 1 {
        quote! {
            #( #iter_terms )*
        }
    } else {
        quote! {
            (#(
                #iter_terms,
            )*)
        }
    };

    // Generate builder calls for each term
    let builder_calls = terms
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let index = i as u32;
            let iter_term = i < iter_terms.len();
            expand_term_builder_calls(t, index, iter_term)
        })
        .collect::<Vec<_>>();

    (iter_type, builder_calls)
}

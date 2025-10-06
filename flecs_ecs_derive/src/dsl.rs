// DSL parsing and expansion utilities for query/system/observer macros.

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    Expr, Ident, LitStr, Path, Result, Token, Type, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token::Bracket,
};

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

fn peek_id(input: &ParseStream) -> bool {
    input.peek(Ident)
        || input.peek(Token![*])
        || input.peek(Token![_])
        || input.peek(Token![$])
        || input.peek(LitStr)
        || input.peek(Token![Self])
}

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

pub struct TermId {
    pub ident: Option<TermIdent>,
    pub trav_self: bool,
    pub trav_up: bool,
    pub up_ident: Option<TermIdent>,
    pub trav_desc: bool,
    pub trav_cascade: bool,
    pub cascade_ident: Option<TermIdent>,
    pub span: Span,
}

impl TermId {
    fn new(ident: Option<TermIdent>, span: Span) -> Self {
        Self {
            ident,
            trav_self: false,
            trav_up: false,
            up_ident: None,
            trav_desc: false,
            trav_cascade: false,
            cascade_ident: None,
            span,
        }
    }
}

fn peek_trav(input: ParseStream) -> bool {
    input.peek(kw::cascade)
        || input.peek(kw::desc)
        || input.peek(kw::up)
        || input.peek(Token![self])
}

impl Parse for TermId {
    fn parse(input: ParseStream) -> Result<Self> {
        let span: Span = input.span();
        let ident = if !peek_trav(input) {
            let ident = input.parse::<TermIdent>()?;
            if input.peek(Token![|]) && !input.peek2(Token![|]) {
                input.parse::<Token![|]>()?;
            }
            Some(ident)
        } else {
            None
        };
        let mut out = Self::new(ident, span);

        while peek_trav(input) {
            if input.peek(kw::cascade) {
                input.parse::<kw::cascade>()?;
                out.trav_cascade = true;

                if input.peek(Ident) || input.peek(Token![$]) {
                    out.cascade_ident = Some(input.parse::<TermIdent>()?);
                }
            }
            if input.peek(kw::desc) {
                input.parse::<kw::desc>()?;
                out.trav_desc = true;
            }
            if input.peek(kw::up) {
                input.parse::<kw::up>()?;
                out.trav_up = true;

                if input.peek(Ident) || input.peek(Token![$]) {
                    out.up_ident = Some(input.parse::<TermIdent>()?);
                }
            }
            if input.peek(Token![self]) {
                input.parse::<Token![self]>()?;
                out.trav_self = true;
            }
            if input.peek(Token![|]) && !input.peek2(Token![|]) {
                input.parse::<Token![|]>()?;
            }
        }

        Ok(out)
    }
}

#[allow(clippy::large_enum_variant)]
pub enum TermType {
    Pair(TermId, TermId),
    Component(TermId),
}

pub struct Term {
    pub access: Access,
    pub reference: Reference,
    pub oper: TermOper,
    pub source: TermId,
    pub ty: TermType,
    pub span: Span,
}

impl Parse for Term {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = input.span();
        let access = input.parse::<Access>()?;
        let oper = input.parse::<TermOper>()?;
        let reference = input.parse::<Reference>()?;
        if peek_id(&input) {
            let initial = input.parse::<TermId>()?;
            if !input.peek(Token![,]) && !input.peek(Token![|]) && !input.is_empty() {
                // Component or pair with explicit source
                let inner;
                parenthesized!(inner in input);
                let source = inner.parse::<TermId>()?;
                if inner.peek(Token![,]) {
                    // Pair
                    inner.parse::<Token![,]>()?;
                    let second = inner.parse::<TermId>()?;
                    Ok(Term {
                        access,
                        reference,
                        oper,
                        source,
                        ty: TermType::Pair(initial, second),
                        span,
                    })
                } else {
                    // Component
                    Ok(Term {
                        access,
                        reference,
                        oper,
                        source,
                        ty: TermType::Component(initial),
                        span,
                    })
                }
            } else {
                // Base case single component identifier
                Ok(Term {
                    access,
                    reference,
                    oper,
                    source: TermId::new(None, input.span()),
                    ty: TermType::Component(initial),
                    span,
                })
            }
        } else {
            // Pair without explicit source
            let inner;
            parenthesized!(inner in input);
            let first = inner.parse::<TermId>()?;
            inner.parse::<Token![,]>()?;
            let second = inner.parse::<TermId>()?;
            Ok(Term {
                access,
                reference,
                oper,
                source: TermId::new(None, input.span()),
                ty: TermType::Pair(first, second),
                span,
            })
        }
    }
}

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
                terms.last_mut().unwrap().oper = TermOper::Or;
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

pub fn expand_type(ident: &TermIdent) -> Option<TokenStream> {
    match ident {
        TermIdent::Type(ty) => Some(quote! { #ty }),
        TermIdent::EnumType(ty) => Some(quote! { #ty }),
        TermIdent::Wildcard => Some(quote! { flecs_ecs::core::flecs::Wildcard }),
        TermIdent::Any => Some(quote! { flecs_ecs::core::flecs::Any }),
        TermIdent::SelfType => Some(quote! { Self }),
        _ => None,
    }
}

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

pub fn expand_dsl(terms: &mut [Term]) -> (TokenStream, Vec<TokenStream>) {
    let mut iter_terms = Vec::new();
    for t in terms.iter() {
        match expand_term_type(t) {
            Some(ty) => iter_terms.push(ty),
            None => break,
        };
    }
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
    let builder_calls = terms
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let index = i as u32;
            let mut ops = Vec::new();
            let mut needs_accessor = false;
            let iter_term = i < iter_terms.len();
            let mut term_accessor = if !iter_term {
                quote! { .term() }
            } else {
                quote! { .term_at(#index) }
            };

            match &t.ty {
                TermType::Pair(first, second) => {
                    let first_id = first.ident.as_ref().expect("Pair with no first.");
                    let second_id = second.ident.as_ref().expect("Pair with no second.");
                    let first_ty = expand_type(first_id);
                    let second_ty = expand_type(second_id);

                    match first_id {
                        TermIdent::Variable(var) => {
                            let var_name = format!("${}", var.value());
                            ops.push(quote! { .set_first(#var_name) });
                        }
                        TermIdent::SelfVar => ops.push(quote! { .set_first(self) }),
                        TermIdent::Local(ident) => ops.push(quote! { .set_first(#ident) }),
                        TermIdent::Literal(lit) => ops.push(quote! { .set_first(#lit) }),
                        _ => {
                            if !iter_term {
                                ops.push(quote! { .set_first(id::<#first_ty>()) });
                            }
                        }
                    };

                    match second_id {
                        TermIdent::Variable(var) => {
                            let var_name = format!("${}", var.value());
                            ops.push(quote! { .set_second(#var_name) });
                        }
                        TermIdent::SelfVar => ops.push(quote! { .set_second(self) }),
                        TermIdent::Local(ident) => ops.push(quote! { .set_second(#ident) }),
                        TermIdent::Literal(lit) => ops.push(quote! { .set_second(#lit) }),
                        _ => {
                            if !iter_term {
                                ops.push(quote! { .set_second(id::<#second_ty>()) });
                            }
                        }
                    };

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
                TermType::Component(term) => {
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
                                term_accessor = quote! { .with_enum(#ty) };
                                needs_accessor = true;
                            }
                        },
                        _ => {
                            if !iter_term {
                                term_accessor = quote! { .with(id::<#ty>()) };
                                needs_accessor = true;
                            }
                        },
                    };

                    // Configure traversal
                    let id_ops = expand_trav(term);
                    if !id_ops.is_empty() {
                        ops.push(quote! { #( #id_ops )* });
                    }
                }
            }

            // Configure source
            if let Some(source) = &t.source.ident {
                let ty = expand_type(source);
                match source {
                    TermIdent::Variable(var) => {
                        let var_name = format!("${}", var.value());
                        ops.push(quote! { .set_src(#var_name) });
                    }
                    TermIdent::SelfVar => ops.push(quote! { .set_src(self) }),
                    TermIdent::Local(ident) => ops.push(quote! { .set_src(#ident) }),
                    TermIdent::Literal(lit) => ops.push(quote! { .set_src(#lit) }),
                    _ => ops.push(quote! { .set_src(id::<#ty>()) }),
                };
            }

            // Configure operator

            if iter_term {
                if !matches!(t.oper, TermOper::And | TermOper::Optional) {
                    ops.push(quote_spanned!{
                        t.span => ; compile_error!("Only 'optional' and 'and' operators allowed for static terms.")
                    });
                }
            } else {
                match &t.oper {
                    TermOper::Not => ops.push(quote! { .not() }),
                    TermOper::Or => ops.push(quote! { .or() }),
                    TermOper::AndFrom => ops.push(quote! { .and_from() }),
                    TermOper::NotFrom => ops.push(quote! { .not_from() }),
                    TermOper::OrFrom => ops.push(quote! { .or_from() }),
                    TermOper::Optional => ops.push(quote! { .optional() }),
                    TermOper::And => {}
                }
            }

            // Configure traversal for source
            let id_ops = expand_trav(&t.source);
            if !id_ops.is_empty() {
                ops.push(quote! { .src() #( #id_ops )* });
            }

            // Configure access
            if iter_term {
                if !matches!(t.access, Access::Omitted | Access::Filter) {
                    ops.push(quote_spanned!{t.span => ; compile_error!("Only [filter] is allowed on static terms.")});
                }

                if t.access == Access::Filter {
                    ops.push(quote! { .filter() });
                }
            } else {
                match &t.reference {
                    Reference::None => {},
                    _ => ops.push(
                        quote_spanned!{
                            t.span => ; compile_error!("Static term located after a dynamic term, re-order such that `&` and `&mut are first.")
                        })
                }

                match &t.access {
                    Access::In => ops.push(quote! { .set_in() }),
                    Access::Out => ops.push(quote! { .set_out() }),
                    Access::InOut => ops.push(quote! { .set_inout() }),
                    Access::Filter => ops.push(quote! { .filter() }),
                    Access::None => ops.push(quote! { .set_inout_none() }),
                    Access::Omitted => {}
                }
            }

            if !ops.is_empty() || needs_accessor {
                Some(quote! {
                    #term_accessor
                    #( #ops )*
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    (iter_type, builder_calls)
}

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
pub(crate) fn expand_query(input: Builder) -> TokenStream {
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

/// Expansion function for the `system` macro.
///
/// Generates a system builder with the appropriate method calls based on the DSL terms.
///
/// # Arguments
///
/// * `input` - A `Builder` struct containing the system name, world, and DSL terms
///
/// # Returns
///
/// A `TokenStream` containing the generated system builder code
pub(crate) fn expand_system(input: Builder) -> TokenStream {
    let mut terms = input.dsl.terms;
    let (iter_type, builder_calls) = expand_dsl(&mut terms);
    let world = input.world;

    match input.name {
        Some(name) => quote! {
            (#world).system_named::<#iter_type>(#name)
            #(
                #builder_calls
            )*
        },
        None => quote! {
            (#world).system::<#iter_type>()
            #(
                #builder_calls
            )*
        },
    }
}

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
pub(crate) fn expand_observer(input: Observer) -> TokenStream {
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

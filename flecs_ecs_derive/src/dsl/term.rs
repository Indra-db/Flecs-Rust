// Term-related structures for DSL parsing

use proc_macro2::Span;
use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
    parenthesized,
};

use super::types::{Access, Reference, TermIdent, TermOper, peek_id, peek_trav};

/// Term identifier with traversal options
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
    pub(crate) fn new(ident: Option<TermIdent>, span: Span) -> Self {
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

impl Parse for TermId {
    fn parse(input: ParseStream) -> Result<Self> {
        use super::types::kw;
        use syn::Ident;
        
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

/// Term type (either a component or a pair)
#[allow(clippy::large_enum_variant)]
pub enum TermType {
    Pair(TermId, TermId),
    Component(TermId),
}

/// A complete term in the DSL
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

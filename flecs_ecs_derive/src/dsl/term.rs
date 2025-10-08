//! Term-related structures for DSL parsing
//!
//! This module defines the structures that represent parsed query terms:
//! - `TermId`: Component identifier with optional traversal
//! - `TermType`: Either a single component or a pair
//! - `Term`: Complete term with all metadata
//!
//! # Term Structure
//!
//! A complete term consists of:
//! ```text
//! [access] operator reference type(source | traversal)
//!   ↓        ↓        ↓         ↓      ↓        ↓
//! [filter]   !        &      Position  up    ChildOf
//! ```
//!
//! # Examples
//!
//! ```ignore
//! // Simple component
//! &Position
//!   → Term { reference: Ref, ty: Component(Position), ... }
//!
//! // Component with traversal
//! &Position(up ChildOf)
//!   → Term { ty: Component(Position { trav_up: true, ... }), ... }
//!
//! // Pair
//! (ChildOf, Parent)
//!   → Term { ty: Pair(ChildOf, Parent), ... }
//!
//! // Complex term
//! [filter] !Tag(source | up Parent)
//!   → Term { access: Filter, oper: Not, source: ..., ... }
//! ```

use proc_macro2::Span;
use syn::{
    Result, Token, parenthesized,
    parse::{Parse, ParseStream},
};

use super::types::{Access, EqualityOper, Reference, TermIdent, TermOper, peek_id, peek_trav};

/// Term identifier with traversal options
///
/// Represents a component identifier along with optional relationship traversal.
/// Traversal controls how the query follows relationships in the entity hierarchy.
///
/// # Fields
///
/// - `ident`: The component identifier (type, variable, etc.)
/// - `trav_self`: Include the entity itself
/// - `trav_up`: Traverse up the hierarchy
/// - `up_ident`: Specific relationship to traverse up
/// - `trav_desc`: Traverse down (descending/breadth-first)
/// - `trav_cascade`: Traverse down (cascading/depth-first)
/// - `cascade_ident`: Specific relationship to cascade
/// - `span`: Source location for error reporting
///
/// # Examples
///
/// ```ignore
/// // Simple identifier
/// Position
///   → TermId { ident: Some(Position), trav_self: false, ... }
///
/// // With up traversal
/// Position(up ChildOf)
///   → TermId { ident: Some(Position), trav_up: true, up_ident: Some(ChildOf), ... }
///
/// // With cascade
/// Position(cascade ChildOf)
///   → TermId { ident: Some(Position), trav_cascade: true, cascade_ident: Some(ChildOf), ... }
///
/// // Combined traversal
/// Position(self up cascade ChildOf)
///   → TermId { trav_self: true, trav_up: true, trav_cascade: true, ... }
/// ```
///
/// # Traversal Semantics
///
/// - `up`: Look for component on parent entities
/// - `cascade`: Depth-first iteration over children
/// - `desc`: Breadth-first iteration over children
/// - `self`: Include current entity (usually combined with traversal)
pub struct TermId {
    /// The component identifier
    pub ident: Option<TermIdent>,
    /// Include the entity itself in traversal
    pub trav_self: bool,
    /// Traverse up the hierarchy
    pub trav_up: bool,
    /// Specific relationship for up traversal
    pub up_ident: Option<TermIdent>,
    /// Traverse down breadth-first
    pub trav_desc: bool,
    /// Traverse down depth-first
    pub trav_cascade: bool,
    /// Specific relationship for cascade traversal
    pub cascade_ident: Option<TermIdent>,
    /// Source span for error reporting
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
///
/// Represents the fundamental type of a query term:
/// - `Component`: A single component type
/// - `Pair`: A relationship pair (First, Second)
///
/// # Examples
///
/// ```ignore
/// // Component
/// Position
///   → TermType::Component(TermId { ident: Some(Position), ... })
///
/// // Pair
/// (ChildOf, Parent)
///   → TermType::Pair(
///         TermId { ident: Some(ChildOf), ... },
///         TermId { ident: Some(Parent), ... }
///     )
///
/// // Wildcard pair
/// (ChildOf, *)
///   → TermType::Pair(
///         TermId { ident: Some(ChildOf), ... },
///         TermId { ident: Some(Wildcard), ... }
///     )
/// ```
///
/// # Pairs in Flecs
///
/// Pairs represent relationships between entities:
/// - (ChildOf, Parent): Entity is a child of Parent
/// - (Likes, Food): Entity likes Food
/// - (*, Target): Any relationship with Target
#[allow(clippy::large_enum_variant)]
pub enum TermType {
    /// Relationship pair: (First, Second)
    Pair(TermId, TermId),
    /// Single component
    Component(TermId),
    /// Equality expression: variable == entity/string
    Equality(EqualityExpr),
}

/// Equality expression for comparing variables
///
/// Represents expressions like `$this == UssEnterprise` or `$this ~= "Uss"`
///
/// # Fields
///
/// - `left`: Left side variable (e.g., `$this`, `$"parent"`)
/// - `oper`: Comparison operator (==, !=, ~=)
/// - `right`: Right side (entity, string, or variable)
///
/// # Examples
///
/// ```ignore
/// $this == UssEnterprise
///   → EqualityExpr {
///       left: Variable("this"),
///       oper: Equal,
///       right: Type(UssEnterprise)
///     }
///
/// $this ~= "Uss"
///   → EqualityExpr {
///       left: Variable("this"),
///       oper: Match,
///       right: Literal("Uss")
///     }
/// ```
pub struct EqualityExpr {
    /// Left side variable
    pub left: TermIdent,
    /// Comparison operator
    pub oper: super::types::EqualityOper,
    /// Right side (entity, string, or variable)
    pub right: TermIdent,
}


/// A complete term in the DSL
///
/// Represents a fully parsed query term with all its attributes:
/// - How it's accessed (access)
/// - Whether it's a reference (reference)
/// - How it combines with other terms (oper)
/// - Where to find the component (source)
/// - What component/pair it refers to (ty)
///
/// # Fields
///
/// - `access`: Access mode ([in], [out], [filter], etc.)
/// - `reference`: Rust reference type (&, &mut, none)
/// - `oper`: Logical operator (!, ?, ||, etc.)
/// - `source`: Optional explicit source entity
/// - `ty`: The component or pair being queried
/// - `span`: Source location for error reporting
///
/// # Examples
///
/// ```ignore
/// // Simple read-only component
/// &Position
///   → Term {
///       access: Omitted,
///       reference: Ref,
///       oper: And,
///       source: TermId { ident: None, ... },
///       ty: Component(Position),
///       span: ...
///     }
///
/// // Filtered component with NOT operator
/// ![filter] Dead
///   → Term {
///       access: Filter,
///       reference: None,
///       oper: Not,
///       ty: Component(Dead),
///       ...
///     }
///
/// // Optional mutable component
/// ?&mut Velocity
///   → Term {
///       reference: Mut,
///       oper: Optional,
///       ty: Component(Velocity),
///       ...
///     }
///
/// // Pair with explicit source
/// (ChildOf, Parent)(source)
///   → Term {
///       source: TermId { ident: Some(source), ... },
///       ty: Pair(ChildOf, Parent),
///       ...
///     }
/// ```
///
/// # Term Order
///
/// Terms with references (&, &mut) must come before filter terms:
/// ```ignore
/// // ✓ Valid
/// query!(world, &Position, &mut Velocity, !Dead, [filter] Active)
///
/// // ✗ Invalid - reference after filter
/// query!(world, Tag, &Position)  // Compile error!
/// ```
pub struct Term {
    /// Access mode: [in], [out], [filter], etc.
    pub access: Access,
    /// Rust reference: &, &mut, or none
    pub reference: Reference,
    /// Logical operator: !, ?, ||, etc.
    pub oper: TermOper,
    /// Optional explicit source entity
    pub source: TermId,
    /// Component or pair being queried
    pub ty: TermType,
    /// Source span for error reporting
    pub span: Span,
}

impl Parse for Term {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = input.span();
        let access = input.parse::<Access>()?;
        let mut oper = input.parse::<TermOper>()?;
        let reference = input.parse::<Reference>()?;
        
        // Check for equality expression: $var == entity or $var != entity or $var ~= "string"
        if input.peek(Token![$]) {
            let lookahead = input.fork();
            let _left = lookahead.parse::<TermIdent>();
            if _left.is_ok() && (lookahead.peek(Token![==]) || lookahead.peek(Token![!=]) 
                || (lookahead.peek(Token![~]) && lookahead.peek2(Token![=]))) {
                // This is an equality expression
                let left = input.parse::<TermIdent>()?;
                
                let equality_oper = if input.peek(Token![==]) {
                    input.parse::<Token![==]>()?;
                    EqualityOper::Equal
                } else if input.peek(Token![!=]) {
                    input.parse::<Token![!=]>()?;
                    // NotEqual should add .not() operator
                    oper = TermOper::Not;
                    EqualityOper::NotEqual
                } else if input.peek(Token![~]) {
                    input.parse::<Token![~]>()?;
                    input.parse::<Token![=]>()?;
                    EqualityOper::Match
                } else {
                    unreachable!()
                };
                
                let right = input.parse::<TermIdent>()?;
                
                // Check if this is a negated match (string starts with '!')
                if equality_oper == EqualityOper::Match {
                    if let TermIdent::Literal(lit) = &right {
                        if lit.value().starts_with('!') {
                            oper = TermOper::Not;
                        }
                    }
                }
                
                return Ok(Term {
                    access,
                    reference,
                    oper,
                    source: TermId::new(None, input.span()),
                    ty: TermType::Equality(EqualityExpr {
                        left,
                        oper: equality_oper,
                        right,
                    }),
                    span,
                });
            }
        }
        
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

// DSL parsing and expansion utilities for query/system/observer macros.
//
// This module provides a declarative syntax for building Flecs queries, systems, and observers.
// The DSL is parsed into structured types and then expanded into the appropriate builder calls.
//
// Module structure:
// - `types`: Core types like Reference, Access, TermIdent, TermOper
// - `term`: Term-related structures (TermId, Term, TermType)
// - `parser`: DSL parser and builder structures (Dsl, Builder, Observer)
// - `ident_expander`: Functions for expanding TermIdent into builder calls
// - `builder`: High-level builder call generation logic
// - `expansion`: Common expansion utilities (traversal, term types)
// - `query`: Query-specific expansion logic
// - `system`: System-specific expansion logic
// - `observer`: Observer-specific expansion logic

mod types;
mod term;
mod parser;
mod ident_expander;
mod builder;
mod expansion;
mod query;
mod system;
mod observer;

// Re-export public API
pub use parser::{Builder, Observer};
pub use query::expand_query;
pub use system::expand_system;
pub use observer::expand_observer;

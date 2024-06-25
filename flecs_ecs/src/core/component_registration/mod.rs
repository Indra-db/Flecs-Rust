//! Contains types and traits that define what a component is and how it is registered.

mod helpers;
mod registration;
pub mod registration_traits;
pub mod registration_types;

pub(crate) use helpers::*;
#[doc(hidden)]
pub use registration::*;
#[doc(hidden)]
pub use registration_traits::*;
#[doc(hidden)]
pub use registration_types::*;

/// Marker component, added to an entity to indicate it is an alert.
pub trait SeverityAlert {}

// Re-export alert tag components from core::flecs::alerts
// Note: AlertComponent is renamed to avoid conflict with the Alert wrapper struct
pub use crate::core::flecs::alerts::{
    AlertComponent, AlertInstance, AlertTimeout, AlertsActive, Critical, Error, Info, Warning,
};

// Implement SeverityAlert for the alert severity types
impl SeverityAlert for Info {}
impl SeverityAlert for Warning {}
impl SeverityAlert for Error {}
impl SeverityAlert for Critical {}

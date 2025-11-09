//! Builtin standard units for quantities like length, time, mass, and more.
//!
//! The units addon provides predefined unit types following standard conventions
//! (SI units, etc.). Unlike core Flecs modules, the units module must be explicitly
//! imported and behaves like an application-defined module. This means entity IDs
//! are not fixed and depend on import order.
//!
//! # Features
//!
//! - **Standard Units**: Length (meters, kilometers), time (seconds, hours), mass, etc.
//! - **Prefixes**: SI prefixes from Yocto (10⁻²⁴) to Yotta (10²⁴)
//! - **Angles**: Degrees, radians
//! - **Temperature**: Kelvin, Celsius, Fahrenheit
//! - **Derived Units**: Force, pressure, frequency, and more
//!
//! # Usage
//!
//! The units module must be explicitly imported:
//!
//! ```
//! use flecs_ecs::prelude::*;
//! use flecs_ecs::addons::units::*;
//!
//! let world = World::new();
//!
//! // Import the units module
//! world.import::<Units>();
//!
//! // Now units are available for use
//! // You can use unit entities in component definitions
//! ```
//!
//! # Available Unit Categories
//!
//! - **Duration**: Seconds, Minutes, Hours, Days
//! - **Length**: Meters (with prefixes), Kilometers, Miles
//! - **Mass**: Grams (with prefixes), Kilograms
//! - **Temperature**: Kelvin, Celsius, Fahrenheit
//! - **Angle**: Radians, Degrees
//! - **Data**: Bits, Bytes (with prefixes like Kibi, Mebi, Gibi)
//! - **Frequency**: Hertz (with prefixes)
//! - **Force**: Newton
//! - **Pressure**: Pascal, Bar
//! - **And many more...**
//!
//! # Note on Entity IDs
//!
//! Because the units module is imported like an application-defined module,
//! the entity IDs generated for units are **not fixed**. They depend on the
//! order in which modules are imported. Always import the units module in
//! the same order across application runs for consistency.
//!
//! # See also
//!
//! - [`Units`] - Module struct for importing units
//! - [`World::import()`] - Import a module

mod types;
pub use types::*;

use super::module::Module;
use crate::core::World;
use flecs_ecs_derive::Component;

#[derive(Clone, Copy, Component, Default)]
pub struct Units;

impl Module for Units {
    fn module(world: &World) {
        unsafe { flecs_ecs_sys::FlecsUnitsImport(world.ptr_mut()) };
    }
}

//! The REST module provides a small REST API for accessing application data remotely.
//!
//! This module uses the HTTP server and JSON serializer to provide access to ECS data,
//! enabling remote inspection, debugging, and monitoring of your Flecs application.
//! The REST API is commonly used with tools like the Flecs Explorer web interface.
//!
//! # Features
//!
//! - **Remote Access**: Query entities, components, and systems over HTTP
//! - **JSON Serialization**: Automatic serialization of ECS data to JSON
//! - **Explorer Integration**: Works seamlessly with the Flecs Explorer web UI
//! - **Real-time Monitoring**: Monitor application state in real-time
//!
//! # Usage
//!
//! Enable the REST server by adding a [`Rest`] component to a singleton entity,
//! or use the [`App::enable_rest()`](crate::addons::app::App::enable_rest) method
//! when using the app addon.
//!
//! # Example
//!
//! ```no_run
//! use flecs_ecs::prelude::*;
//!
//! let world = World::new();
//!
//! // Enable REST API on port 27750 (default)
//! world.set(flecs::rest::Rest::default());
//!
//! // Or use the app addon for simpler setup
//! world.app()
//!     .enable_rest(27750)
//!     .run();
//! ```
//!
//! Once enabled, you can access the REST API at `http://localhost:27750`.
//!
//! # API Reference
//!
//! For detailed REST API endpoint documentation and usage, see the
//! [Flecs REST API Manual](https://www.flecs.dev/flecs/md_docs_2RestApi.html).
//!
//! # See also
//!
//! - [`Rest`] - Component to enable the REST server
//! - [`App::enable_rest()`](crate::addons::app::App::enable_rest) - Enable REST via the app addon
//! - [Flecs Explorer](https://www.flecs.dev/explorer) - Web-based ECS inspector
//! - [REST API Manual](https://www.flecs.dev/flecs/md_docs_2FlecsRemoteApi.html)

use super::*;
// REST module components
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rest {
    #[doc = "< Port of server (optional, default = 27750)"]
    pub port: u16,
    #[doc = "< Interface address (optional, default = 0.0.0.0)"]
    pub ipaddr: *mut ::core::ffi::c_char,
    pub impl_: *mut ::core::ffi::c_void,
}

impl Default for Rest {
    fn default() -> Self {
        Self {
            port: Default::default(),
            ipaddr: core::ptr::null_mut::<core::ffi::c_char>(),
            impl_: core::ptr::null_mut::<core::ffi::c_void>(),
        }
    }
}

impl_component_traits_binding_type_w_id!(Rest, ECS_REST);
unsafe impl Send for Rest {}
unsafe impl Sync for Rest {}

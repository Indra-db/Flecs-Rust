#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_macros)]

pub mod core {
    pub mod archetype;
    pub mod c_binding;
    pub mod c_types;
    pub mod component;
    pub mod component_ref;
    pub mod component_registration;
    pub mod entity;
    pub mod entity_view;
    pub mod enum_type;
    pub mod filter;
    pub mod id;
    pub mod iterable;
    pub mod lifecycle_traits;
    pub mod scoped_world;
    pub mod table;
    pub mod term;
    pub mod world;
    pub mod utility {
        pub mod errors;
        pub mod functions;
        pub mod log;
        pub mod macros;
    }
    pub mod data_structures {
        pub mod pair;
    }
}

pub mod addons {}

use core::component_registration::*;

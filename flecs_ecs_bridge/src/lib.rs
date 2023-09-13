#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_macros)]

pub mod core {
    pub mod c_binding;
    pub mod c_types;
    pub mod component;
    pub mod entity;
    pub mod id;
    pub mod lifecycle_traits;
    pub mod world;
    pub mod utility {
        pub mod errors;
        pub mod functions;
    }
}

pub mod addons {}

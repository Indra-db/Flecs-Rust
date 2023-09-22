#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_macros)]

pub mod core {
    pub mod archetype;
    pub mod c_binding;
    pub mod c_types;
    pub mod component;
    pub mod entity;
    pub mod enum_type;
    pub mod id;
    pub mod lifecycle_traits;
    pub mod table;
    pub mod world;
    pub mod utility {
        pub mod errors;
        pub mod functions;
    }
    pub mod data_structures {
        pub mod pair;
    }
}

pub mod addons {}

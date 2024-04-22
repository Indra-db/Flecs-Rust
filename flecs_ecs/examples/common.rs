#![allow(dead_code)]
#![allow(unused_imports)]

use std::ffi::c_void;

pub use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Mass {
    pub value: f32,
}

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
pub struct Apples;

#[derive(Component)]
pub struct Pears;

#[derive(Component)]
pub struct Walking;

#[derive(Component)]
pub struct Tag;

#[derive(Component)]
pub struct Human;

#[derive(Component)]
pub struct Attack {
    pub value: f32,
}

#[derive(Component)]
pub struct Defence {
    pub value: f32,
}

#[derive(Component)]
pub struct Damage {
    pub value: f32,
}

#[derive(Component)]
pub struct FreightCapacity {
    pub value: f32,
}

#[derive(Component)]
pub struct ImpulseSpeed {
    pub value: f32,
}

#[derive(Component)]
pub struct HasFlt;

#[derive(Component)]
pub struct First;

#[derive(Component)]
pub struct Second;

#[derive(Component)]
pub struct Third;

#[derive(Component)]
pub struct Group;

#[allow(dead_code)]
fn main() {
    //this file is for common structs and functions used in the examples
}

#[macro_export]
macro_rules! fprintln {
    ($str_vec:expr) => {
        {
            $str_vec.push(format!("\n"));
            println!();
        }
    };
    ($str_vec:expr, $format_string:expr) => {
        {
            $str_vec.push(format!($format_string));
            println!($format_string);
        }
    };
    ($str_vec:expr, $format_string:expr, $($arg:expr),*) => {
        {
            $str_vec.push(format!($format_string, $($arg),*));
            println!($format_string, $($arg),*);
        }
    };
}

pub struct Snap {
    pub str: Vec<String>,
}

impl Snap {
    pub fn setup_snapshot_test() -> Snap {
        Snap { str: Vec::new() }
    }

    pub fn cvoid(&self) -> *mut c_void {
        self as *const Snap as *mut c_void
    }

    pub fn push(&mut self, str: String) {
        self.str.push(str);
    }

    pub fn count(&self) -> usize {
        self.str.len()
    }

    #[allow(clippy::mut_from_ref)]
    pub fn from<'a>(it: &'a flecs_ecs::core::Iter) -> &'a mut Snap {
        unsafe { it.context::<Snap>() }
    }

    pub fn test(&self) {
        insta::with_settings!({filters => vec![
            (r"id: (\d+)\s", "[ID] ")
        ]}, {
            insta::assert_yaml_snapshot!(self.str);

        });
    }
}

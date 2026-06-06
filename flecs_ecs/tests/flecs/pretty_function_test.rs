#![allow(dead_code)]
#![allow(unused_imports)]
use crate::common_test::*;
use flecs_ecs::prelude::*;

// The C++ PrettyFunction tests print ECS_FUNC_NAME (compiler-specific function name macro)
// and do compile-time validation of enum constants. In Rust, we use core::any::type_name
// for the type name, and the flecs enum reflection validates constants at registration time.

#[test]
fn pretty_function_component() {
    // C++ prints ECS_FUNC_NAME for pretty_type<Position>() and asserts true.
    // Rust equivalent: print the type name and assert trivially.
    let type_name = core::any::type_name::<Position>();
    // Just verify it contains "Position"
    assert!(type_name.contains("Position"));
}

// TODO: missing API: pretty_function_enum — C++ uses flecs::_::enum_constant_is_valid<E,C>()
// compile-time checks, which have no direct public Rust equivalent in the flecs-rust bindings.
// The C++ test verifies:
//   - flecs::_::enum_constant_is_valid<Color, Color::Red>() == true
//   - flecs::_::enum_constant_is_valid<Color, (Color)3>() == false
//   - Template enum types with commas in names are supported
// In Rust, enum constant validity is checked at component registration time via proc-macro
// reflection, but there's no public flecs_ecs::_::enum_constant_is_valid API.
// #[test]
// fn pretty_function_enum() { ... }

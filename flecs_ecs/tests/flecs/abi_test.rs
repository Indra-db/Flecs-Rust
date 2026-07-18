//! Guards against layout drift between the generated bindings and the
//! compiled C library for structs with FLECS_DEBUG-gated fields.

use flecs_ecs::sys;

#[test]
fn debug_gated_struct_sizes_match_compiled_c() {
    unsafe {
        assert_eq!(
            core::mem::size_of::<sys::ecs_ref_t>(),
            sys::ecs_rust_sizeof_ecs_ref_t(),
            "ecs_ref_t layout differs between bindings and compiled C"
        );
        assert_eq!(
            core::mem::size_of::<sys::ecs_map_t>(),
            sys::ecs_rust_sizeof_ecs_map_t(),
            "ecs_map_t layout differs between bindings and compiled C"
        );
        assert_eq!(
            core::mem::size_of::<sys::ecs_map_iter_t>(),
            sys::ecs_rust_sizeof_ecs_map_iter_t(),
            "ecs_map_iter_t layout differs between bindings and compiled C"
        );
        assert_eq!(
            core::mem::size_of::<sys::ecs_stack_t>(),
            sys::ecs_rust_sizeof_ecs_stack_t(),
            "ecs_stack_t layout differs between bindings and compiled C"
        );
        assert_eq!(
            core::mem::size_of::<sys::ecs_stack_cursor_t>(),
            sys::ecs_rust_sizeof_ecs_stack_cursor_t(),
            "ecs_stack_cursor_t layout differs between bindings and compiled C"
        );
    }
}

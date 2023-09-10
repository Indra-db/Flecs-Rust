//! Considerations on implementation for user experience vs performance.:
//!
//! 1. Unlike C++, Rust doesn't support compile-time checks for trivial types.
//! 2. Current implementation prioritizes simplicity over performance optimizations.
//!    - If trivial type registration incurs a significant performance penalty, reconsider this approach.
//!
//! Challenges:
//! - Rust lacks several features for this scenario:
//!   a) Trait specialization.
//!   b) Compile-time trivial type checks.
//!   c) A direct equivalent of `placement_new` from C++.
//!      ptr::write still constructs the object on the stack and then moves it, barring optimizations.
//!
//! Potential Solutions:
//! - Bypass the need for `placement_new` with a `placement_ctor` function.
//!   - Drawback: Each field needs manual setting, which impacts user experience.
//!      - example code:
//!      ```ignore
//!           struct MyType {
//!               vec: Vec<i32>,
//!           }
//!
//!           trait PlacementNew {
//!               unsafe fn placement_new(ptr: *mut Self);
//!           }
//!
//!           impl PlacementNew for MyType {
//!               unsafe fn placement_new(ptr: *mut Self) {
//!                   (*ptr).vec = Vec::<i32>::default();
//!               }
//!           }
//!      ```
//! - For potential type optimizations, consider:
//!   a) Utilizing the `Zeroable` trait and rely on user's proper implementation.
//!   b) Implement pseudo-trait specialization, as detailed in:
//!      - http://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html
//!      - https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=1e548abff8e35b97b25adcacdddaacda
//!
//! possible helpful crates for trait specialization / type specialization:
//! - For type casting: https://crates.io/crates/castaway
//!
//! Note: C does the same, where the user needs to opt in for non trivial types. We can do the same.
//! Note2: zerobit pattern

use crate::core::c_binding::bindings::*;
use crate::core::c_types::*;
use std::{ffi::c_void, ptr};

pub fn register_lifecycle_actions<T: Clone + Default>(
    world: *mut ecs_world_t,
    component: ecs_entity_t,
) {
    let mut cl = ecs_type_hooks_t {
        ctor: Some(generic_ctor::<T>),
        dtor: Some(generic_dtor::<T>),
        copy: Some(generic_copy::<T>),
        move_: Some(generic_move::<T>),
        copy_ctor: None,
        move_ctor: None,
        ctor_move_dtor: None,
        move_dtor: None,
        on_add: None,
        on_set: None,
        on_remove: None,
        ctx: std::ptr::null_mut(),
        binding_ctx: std::ptr::null_mut(),
        ctx_free: None,
        binding_ctx_free: None,
    };

    unsafe {
        ecs_set_hooks_id(world, component, &mut cl);
    }
}

extern "C" fn generic_ctor<T: Default>(
    ptr: *mut c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    #[cfg(feature = "enable_core_asserts")]
    assert!(unsafe { (*_type_info).size == std::mem::size_of::<T>() as i32 });

    let arr = ptr as *mut T;
    for i in 0..count as isize {
        unsafe {
            std::ptr::write(arr.offset(i), T::default());
        }
    }
}

extern "C" fn generic_dtor<T>(ptr: *mut c_void, count: i32, _type_info: *const ecs_type_info_t) {
    #[cfg(feature = "enable_core_asserts")]
    assert!(unsafe { (*_type_info).size == std::mem::size_of::<T>() as i32 });
    let arr = ptr as *mut T;
    for i in 0..count as isize {
        unsafe {
            let item = arr.offset(i);
            ptr::drop_in_place(item);
        }
    }
}

extern "C" fn generic_copy<T: Default + Clone>(
    dst_ptr: *mut c_void,
    src_ptr: *const c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    #[cfg(feature = "enable_core_asserts")]
    assert!(unsafe { (*_type_info).size == std::mem::size_of::<T>() as i32 });
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *const T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we're cloning the internal data
        unsafe {
            let src_value = &*(src_arr.offset(i));
            let cloned_value = src_value.clone();
            ptr::write(dst_arr.offset(i), cloned_value);
        }
    }
}

extern "C" fn generic_move<T: Default>(
    dst_ptr: *mut c_void,
    src_ptr: *mut c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    #[cfg(feature = "enable_core_asserts")]
    assert!(unsafe { (*_type_info).size == std::mem::size_of::<T>() as i32 });
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *mut T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we are just moving the internal data around
        unsafe {
            // Leave the source in a default (empty) state, not dropping the previous
            // allocated memory it might hold
            let moved_value = std::ptr::replace(src_arr.offset(i), T::default());
            // Write moved src to dst without dropping src since src is being moved to dst
            ptr::write(dst_arr.offset(i), moved_value);
        }
    }
}

mod tests {
    use super::*;

    #[derive(Default, Debug, Clone)]
    struct MyType {
        vec: Vec<i32>,
        text: String,
        value: i32,
    }

    #[test]
    fn test_generic_move() {
        let mut original = MyType {
            vec: vec![0, 1, 2, 3],
            text: "original".to_string(),
            value: 42,
        };
        let mut moved_to: MyType = Default::default();

        let orig_ptr_before_move = original.vec.as_ptr();
        let moved_to_ptr_before_move = moved_to.vec.as_ptr();

        generic_move::<MyType>(
            &mut moved_to as *mut _ as *mut c_void,
            &mut original as *mut _ as *mut c_void,
            1,
            std::ptr::null(),
        );

        assert_eq!(original.vec, Vec::<i32>::new()); // Original should be default after move
        assert_eq!(moved_to.vec, vec![0, 1, 2, 3]); // Moved_to should have original's values
        assert_eq!(original.text, String::new());
        assert_eq!(moved_to.text, "original");
        assert_eq!(original.value, 0);
        assert_eq!(moved_to.value, 42);

        // The pointers should have been swapped
        assert_eq!(original.vec.as_ptr(), moved_to_ptr_before_move);
        assert_eq!(moved_to.vec.as_ptr(), orig_ptr_before_move);
    }

    #[test]
    fn test_modify_moved_to() {
        let mut original = MyType {
            vec: vec![0, 1, 2, 3],
            text: "original".to_string(),
            value: 42,
        };
        let mut moved_to: MyType = Default::default();

        generic_move::<MyType>(
            &mut moved_to as *mut _ as *mut c_void,
            &mut original as *mut _ as *mut c_void,
            1,
            std::ptr::null(),
        );

        moved_to.vec.push(4);
        moved_to.text.push_str("_modified");
        moved_to.value += 10;

        assert_eq!(original.vec, Vec::<i32>::new()); // Original should be default
        assert_eq!(moved_to.vec, vec![0, 1, 2, 3, 4]); // Moved_to should have new value
        assert_eq!(original.text, String::new());
        assert_eq!(moved_to.text, "original_modified");
        assert_eq!(original.value, 0);
        assert_eq!(moved_to.value, 52);
    }

    #[test]
    fn test_generic_copy() {
        let original = MyType {
            vec: vec![0, 1, 2, 3],
            text: "original".to_string(),
            value: 42,
        };
        let mut copied_to: MyType = Default::default();

        let orig_ptr_before_copy = original.vec.as_ptr();

        generic_copy::<MyType>(
            &mut copied_to as *mut _ as *mut c_void,
            &original as *const _ as *const c_void,
            1,
            std::ptr::null(),
        );

        assert_eq!(original.vec, vec![0, 1, 2, 3]); // Original should remain unchanged
        assert_eq!(copied_to.vec, vec![0, 1, 2, 3]); // copied_to should have original's values
        assert_eq!(original.text, "original");
        assert_eq!(copied_to.text, "original");
        assert_eq!(original.value, 42);
        assert_eq!(copied_to.value, 42);

        // The pointers should be different
        assert_ne!(original.vec.as_ptr(), copied_to.vec.as_ptr());
        assert_eq!(original.vec.as_ptr(), orig_ptr_before_copy);
    }

    #[test]
    fn test_modify_copied_to() {
        let original = MyType {
            vec: vec![0, 1, 2, 3],
            text: "original".to_string(),
            value: 42,
        };
        let mut copied_to: MyType = Default::default();

        generic_copy::<MyType>(
            &mut copied_to as *mut _ as *mut c_void,
            &original as *const _ as *const c_void,
            1,
            std::ptr::null(),
        );

        copied_to.vec.push(4);
        copied_to.text.push_str("_modified");
        copied_to.value += 10;

        assert_eq!(original.vec, vec![0, 1, 2, 3]); // Original should remain unchanged
        assert_eq!(copied_to.vec, vec![0, 1, 2, 3, 4]); // copied_to should have the new value
        assert_eq!(original.text, "original");
        assert_eq!(copied_to.text, "original_modified");
        assert_eq!(original.value, 42);
        assert_eq!(copied_to.value, 52);
    }
}

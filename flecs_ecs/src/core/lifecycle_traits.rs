//! Implementation for lifecycle actions.
//!
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
//!      `ptr::write` still constructs the object on the stack and then moves it, barring optimizations.
//!
//! Potential Solutions:
//! - Bypass the need for `placement_new` with a `placement_ctor` function.
//!   - Drawback: Each field needs manual setting, which impacts user experience.
//!      - example code:
#![cfg_attr(doctest, doc = " ````no_test")]
//!      ```
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
//!      - <http://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html/>
//!      - <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=1e548abff8e35b97b25adcacdddaacda/>
//!
//! possible helpful crates for trait specialization / type specialization:
//! - For type casting: <https://crates.io/crates/castaway/>
//!
//! Note: C does the same, where the user needs to opt in for non trivial types. We can do the same.
//! Note2: zerobit pattern
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::{core::c_types::TypeHooksT, ecs_assert, sys::ecs_type_info_t};
use std::{ffi::c_void, mem::MaybeUninit, ptr};

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn register_lifecycle_actions<T: Default>(type_hooks: &mut TypeHooksT) {
    type_hooks.ctor = Some(generic_ctor::<T>);
    type_hooks.dtor = Some(generic_dtor::<T>);
    type_hooks.move_ = Some(generic_move::<T>);
    type_hooks.move_ctor = Some(generic_move::<T>); //same implementation as move
    type_hooks.ctor_move_dtor = Some(generic_ctor_move_dtor::<T>);
    type_hooks.move_dtor = Some(generic_ctor_move_dtor::<T>); //same implementation as ctor_move_dtor
}

pub fn register_copy_lifecycle_action<T: Clone>(type_hooks: &mut TypeHooksT) {
    type_hooks.copy = Some(generic_copy::<T>);
    type_hooks.copy_ctor = Some(generic_copy::<T>); //same implementation as copy
}

pub fn register_copy_lifecycle_panic_action<T>(type_hooks: &mut TypeHooksT) {
    type_hooks.copy = Some(generic_copy_panic::<T>);
    type_hooks.copy_ctor = Some(generic_copy_panic::<T>); //same implementation as copy
}

/// This is the generic constructor for trivial types
/// It will initialize the memory with the default value of the type
///
/// # Safety
///
/// Can't coexist with T(Entity) or T(World, Entity)
///
/// # Arguments
///
/// * `ptr` - pointer to the memory to be initialized
/// * `count` - number of elements to be initialized
/// * `_type_info` - type info for the type to be initialized
///
/// # See also
///
/// * C++ API: `ctor_impl`
#[doc(alias = "ctor_impl")]
unsafe extern "C" fn generic_ctor<T: Default>(
    ptr: *mut c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );

    let arr = ptr as *mut MaybeUninit<T>;
    for i in 0..count as usize {
        unsafe {
            MaybeUninit::write(&mut *arr.add(i), T::default());
        }
    }
}

/// This is the generic destructor for trivial types
/// It will drop the memory
///
/// # See also
///
/// * C++ API: `dtor_impl`
#[doc(alias = "dtor_impl")]
unsafe extern "C" fn generic_dtor<T>(
    ptr: *mut c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let arr = ptr as *mut T;
    for i in 0..count as isize {
        unsafe {
            let item = arr.offset(i);
            ptr::drop_in_place(item);
        }
    }
}

/// This is the generic copy for trivial types
/// It will copy the memory
///
/// # See also
///
/// * C++ API: `copy_impl`
#[doc(alias = "copy_impl")]
unsafe extern "C" fn generic_copy<T: Clone>(
    dst_ptr: *mut c_void,
    src_ptr: *const c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *const T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we're cloning the internal data
        unsafe {
            let src_value = &*(src_arr.offset(i));
            let dst_value = &mut *dst_arr.offset(i); // Obtain a mutable reference to the destination.
            *dst_value = src_value.clone(); // Assign the cloned value, which automatically drops the previous value.
        }
    }
}

/// This is the generic copy for trivial types
/// It will copy the memory
///
/// # See also
///
/// * C++ API: `copy_impl`
#[doc(alias = "copy_impl")]
extern "C" fn generic_copy_panic<T>(
    _dst_ptr: *mut c_void,
    _src_ptr: *const c_void,
    _count: i32,
    _type_info: *const ecs_type_info_t,
) {
    panic!("Clone is not implemented for type {} and it's being used in a copy / duplicate operation such as component overriding or duplicating entities / components", std::any::type_name::<T>());
}

/// This is the generic move for non-trivial types
/// It will move the memory
///
/// # See also
///
/// * C++ API: `move_impl`
#[doc(alias = "move_impl")]
unsafe extern "C" fn generic_move<T: Default>(
    dst_ptr: *mut c_void,
    src_ptr: *mut c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *mut T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we are just moving the internal data around
        unsafe {
            // Leave the source in a default (empty) state, not dropping the previous
            // allocated memory it might hold
            let moved_value = std::ptr::replace(src_arr.offset(i), T::default());
            let dst_value = &mut *dst_arr.offset(i); // Obtain a mutable reference to the destination.
                                                     // Write moved src to dst without dropping src since src is being moved to dst
            *dst_value = moved_value; // Assign the moved value, which automatically drops the previous value.
        }
    }
}

// TODO: improve this so we can avoid the heap allocation
/// when the struct is non trivial, this will move the value and replace it with a default (heap allocation) and then drop it (deallocating the heap allocation)
///
/// # See also
///
/// * C++ API: `move_ctor_impl`
#[doc(alias = "move_ctor_impl")]
unsafe extern "C" fn generic_ctor_move_dtor<T: Default>(
    dst_ptr: *mut c_void,
    src_ptr: *mut c_void,
    count: i32,
    _type_info: *const ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *mut T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we are just moving the internal data around
        unsafe {
            let moved_value = std::ptr::replace(src_arr.offset(i), T::default());
            let dst_value = &mut *dst_arr.offset(i); // Obtain a mutable reference to the destination.
                                                     // Write moved src to dst without dropping src since src is being moved to dst
            *dst_value = moved_value; // Assign the moved value, which automatically drops the previous value.

            ptr::drop_in_place(src_arr.offset(i));

            //TODO evaluate if this could under here could potentially improve performance
            //my suspicion is that it's dangerous to do this because it could lead to double free / premature free
            {
                //// Read out the source value, effectively moving it.
                //let moved_value = std::ptr::read(src_arr.offset(i));
                //
                //// Write the moved value to the destination.
                //std::ptr::write(dst_arr.offset(i), moved_value);
            }
        }
    }
}

unsafe fn check_type_info<T>(_type_info: *const ecs_type_info_t) -> bool {
    if !_type_info.is_null() {
        unsafe { (*_type_info).size == std::mem::size_of::<T>() as i32 }
    } else {
        true
    }
}

mod tests {
    #![allow(unused_imports)]
    use crate::core::lifecycle_traits::{generic_copy, generic_move};
    use std::os::raw::c_void;

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

        unsafe {
            generic_move::<MyType>(
                &mut moved_to as *mut _ as *mut c_void,
                &mut original as *mut _ as *mut c_void,
                1,
                std::ptr::null(),
            );
        }

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

        unsafe {
            generic_move::<MyType>(
                &mut moved_to as *mut _ as *mut c_void,
                &mut original as *mut _ as *mut c_void,
                1,
                std::ptr::null(),
            );
        }

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

        let original_vec_ptr = original.vec.as_ptr();

        unsafe {
            generic_copy::<MyType>(
                &mut copied_to as *mut _ as *mut c_void,
                &original as *const _ as *const c_void,
                1,
                std::ptr::null(),
            );
        }

        assert_eq!(original.vec, vec![0, 1, 2, 3]); // Original should remain unchanged
        assert_eq!(copied_to.vec, vec![0, 1, 2, 3]); // copied_to should have original's values
        assert_eq!(original.text, "original");
        assert_eq!(copied_to.text, "original");
        assert_eq!(original.value, 42);
        assert_eq!(copied_to.value, 42);

        // The pointers should be different
        assert_ne!(original.vec.as_ptr(), copied_to.vec.as_ptr());
        assert_eq!(original.vec.as_ptr(), original_vec_ptr);
        assert_ne!(original.text.as_ptr(), copied_to.text.as_ptr());
    }

    #[test]
    fn test_modify_copied_to() {
        let original = MyType {
            vec: vec![0, 1, 2, 3],
            text: "original".to_string(),
            value: 42,
        };
        let mut copied_to: MyType = Default::default();

        unsafe {
            generic_copy::<MyType>(
                &mut copied_to as *mut _ as *mut c_void,
                &original as *const _ as *const c_void,
                1,
                std::ptr::null(),
            );
        }

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

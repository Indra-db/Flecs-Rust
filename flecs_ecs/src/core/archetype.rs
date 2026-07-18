//! An [`Archetype`] type can be used to describe what types of components an entity has.

use core::{
    ffi::CStr,
    fmt::{Debug, Display},
    ptr::NonNull,
};

use crate::core::*;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::string::String;

/// An archetype is a vector of component [ids](Id) which can be requested from [entities] or [tables].
///
/// # See also
///
/// * [`EntityView::archetype()`]
/// * [`Table::archetype()`]
///
/// [entities]: EntityView::archetype
/// [tables]: Table::archetype
pub struct Archetype<'a> {
    world: WorldRef<'a>,
    type_vec: &'a [Id],
    _lock: Option<TableLock<'a>>,
}

impl Display for Archetype<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(s) = self.to_string() {
            write!(f, "{s}")
        } else {
            write!(f, "empty archetype")
        }
    }
}

impl Debug for Archetype<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(s) = self.to_string() {
            write!(f, "{s}")
        } else {
            write!(f, "empty archetype")
        }
    }
}

impl<'a> Archetype<'a> {
    pub(crate) fn new(world: impl WorldProvider<'a>, type_vec: &'a [Id]) -> Self {
        Archetype {
            world: world.world(),
            type_vec,
            _lock: None,
        }
    }

    pub(crate) fn new_locked(
        world: impl WorldProvider<'a>,
        type_vec: &'a [Id],
        lock: TableLock<'a>,
    ) -> Self {
        Archetype {
            world: world.world(),
            type_vec,
            _lock: Some(lock),
        }
    }

    /// Convert type to comma-separated string
    ///
    /// # Returns
    ///
    /// `Some(String)` - if the type is not null. `None` - if the type is null.
    pub fn to_string(&self) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_type_str(
                self.world.world_ptr_mut(),
                &sys::ecs_type_t {
                    array: self.type_vec.as_ptr() as *mut _,
                    count: self.type_vec.len() as i32,
                },
            )
        })
        .map(|s| unsafe {
            // Copy the C string into a Rust-allocated String, then free the C
            // allocation with the flecs allocator it came from. Wrapping the C
            // pointer in a Vec would free it with the Rust global allocator,
            // which is undefined behavior when the allocators differ.
            let string = CStr::from_ptr(s.as_ptr()).to_string_lossy().into_owned();
            sys::ecs_os_api.free_.expect("os api is missing")(s.as_ptr() as *mut core::ffi::c_void);
            string
        })
    }

    /// Return the number of elements in the type.
    pub fn count(&self) -> usize {
        self.type_vec.len()
    }

    /// Return a slice to the array of types.
    ///
    /// # Returns
    ///
    /// `Some(&[Id])` - A slice to the array if the type is not null and has elements.
    /// `None` - If the type is null or has no elements.
    ///
    /// # Safety
    ///
    /// This method is safe as long as the underlying array pointed to by `type_vec` does not change
    /// while the slice is in use and the elements are valid [`Id`] instances. The caller must ensure
    /// that the `EcsType` instance (or the underlying `type_vec` data it points to) lives at least as
    /// long as the returned slice to avoid dangling references.
    pub fn as_slice(&self) -> &[Id] {
        self.type_vec
    }

    /// Get [id](IdView) at specified index in type
    ///
    /// # Returns
    ///
    /// Result returned as `IdView`.
    ///
    /// - [`Some(IdView)`] - if the type is not null and the index is within bounds.
    /// - `None` - if the type is null or the index is out of bounds.
    ///
    /// # See also
    ///
    /// [`Some(IdView)`]: IdView
    pub fn get(&self, index: usize) -> Option<IdView<'_>> {
        if index < self.count() {
            Some(IdView::new_from_id(self.world, self.type_vec[index]))
        } else {
            None
        }
    }
}

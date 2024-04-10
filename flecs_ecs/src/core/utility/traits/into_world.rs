use std::{ffi::c_void, marker::PhantomData, ops::Deref, ptr::NonNull};

use flecs_ecs_sys::ecs_get_world;

use crate::core::{Entity, EntityView, Id, Iterable, Query, World, WorldT};

pub trait IntoWorld<'a> {
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world().raw_world.as_ptr()
    }
    #[inline]
    #[doc(hidden)]
    fn world_ptr(&self) -> *const WorldT {
        self.world().raw_world.as_ptr()
    }

    fn world(&self) -> WorldRef<'a>;
}

impl<'a> IntoWorld<'a> for Id<'a> {
    #[inline]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for Entity<'a> {
    #[inline]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for EntityView<'a> {
    #[inline]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a, T> IntoWorld<'a> for Query<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for &'a World {
    #[inline]
    fn world(&self) -> WorldRef<'a> {
        (*self).into()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct WorldRef<'a> {
    raw_world: NonNull<WorldT>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> WorldRef<'a> {
    pub fn real_world(&self) -> WorldRef<'a> {
        unsafe {
            WorldRef::from_ptr(ecs_get_world(self.world_ptr_mut() as *const c_void) as *mut WorldT)
        }
    }

    /// # Safety
    /// Caller must ensure `raw_world` points to a valid `WorldT`
    pub unsafe fn from_ptr(raw_world: *mut WorldT) -> Self {
        WorldRef {
            raw_world: NonNull::new_unchecked(raw_world),
            _marker: PhantomData,
        }
    }
}

impl<'a> From<&'a World> for WorldRef<'a> {
    fn from(world: &'a World) -> Self {
        WorldRef {
            raw_world: world.raw_world,
            _marker: PhantomData,
        }
    }
}

impl<'a> From<&'a *mut WorldT> for &WorldRef<'a> {
    fn from(value: &'a *mut WorldT) -> Self {
        debug_assert_eq!(
            std::mem::size_of::<&*mut WorldT>(),
            std::mem::size_of::<&WorldRef>()
        );
        let before = *value;
        let result = unsafe { std::mem::transmute::<&'a *mut WorldT, &WorldRef>(value) };
        let after = result.raw_world.as_ptr();
        debug_assert_eq!(before, after);
        result
    }
}

impl<'a> Deref for WorldRef<'a> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        debug_assert_eq!(
            std::mem::size_of::<WorldRef>(),
            std::mem::size_of::<World>()
        );
        let before = self.raw_world.as_ptr();
        let result = unsafe { std::mem::transmute::<&WorldRef, &World>(self) };
        let after = result.raw_world.as_ptr();
        debug_assert_eq!(before, after);
        result
    }
}

impl<'a> IntoWorld<'a> for WorldRef<'a> {
    fn world(&self) -> WorldRef<'a> {
        *self
    }
}

impl<'a> IntoWorld<'a> for &WorldRef<'a> {
    fn world(&self) -> WorldRef<'a> {
        **self
    }
}

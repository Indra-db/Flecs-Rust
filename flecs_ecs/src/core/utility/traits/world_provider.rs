use crate::core::*;
use crate::sys;
use core::{ffi::c_void, marker::PhantomData, ops::Deref, ptr::NonNull};

/// Implementations of this trait can provide a reference to a world to operations
/// needing a world. This allows for easily extracting the world from things that
/// already have a reference to one.
pub trait WorldProvider<'a> {
    #[doc(hidden)]
    #[inline(always)]
    fn world_ptr_mut(&self) -> *mut sys::ecs_world_t {
        self.world().raw_world.as_ptr()
    }
    #[doc(hidden)]
    #[inline(always)]
    fn world_ptr(&self) -> *const sys::ecs_world_t {
        self.world().raw_world.as_ptr()
    }

    fn world(&self) -> WorldRef<'a>;
}

impl<'a> WorldProvider<'a> for *mut sys::ecs_world_t {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr(*self) }
    }
}

impl<'a> WorldProvider<'a> for IdView<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> WorldProvider<'a> for EntityView<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a, T: ComponentId> WorldProvider<'a> for Component<'a, T> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.base.entity.world
    }
}

impl<'a> WorldProvider<'a> for UntypedComponent<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> WorldProvider<'a> for &'a World {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        (*self).into()
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct WorldRef<'a> {
    raw_world: NonNull<sys::ecs_world_t>,
    components: NonNull<FlecsIdMap>,
    pub(crate) components_array: NonNull<FlecsArray>,
    _marker: PhantomData<&'a ()>,
}

unsafe impl Send for WorldRef<'_> {}

impl<'a> WorldRef<'a> {
    #[inline(always)]
    pub fn real_world(&self) -> WorldRef<'a> {
        unsafe {
            WorldRef::from_ptr(
                sys::ecs_get_world(self.world_ptr_mut() as *const c_void) as *mut sys::ecs_world_t
            )
        }
    }

    /// # Safety
    /// Caller must ensure `raw_world` points to a valid `sys::ecs_world_t`
    #[inline(always)]
    pub unsafe fn from_ptr(raw_world: *mut sys::ecs_world_t) -> Self {
        unsafe {
            WorldRef {
                raw_world: NonNull::new_unchecked(raw_world),
                components: NonNull::new_unchecked(World::get_components_map_ptr(raw_world)),
                components_array: NonNull::new_unchecked(World::get_components_array_ptr(
                    raw_world,
                )),
                _marker: PhantomData,
            }
        }
    }
}

impl<'a> From<&'a World> for WorldRef<'a> {
    #[inline(always)]
    fn from(world: &'a World) -> Self {
        WorldRef {
            raw_world: world.raw_world,
            components: world.components,
            components_array: world.components_array,
            _marker: PhantomData,
        }
    }
}

impl<'a> From<&'a mut World> for WorldRef<'a> {
    #[inline(always)]
    fn from(world: &'a mut World) -> Self {
        WorldRef {
            raw_world: world.raw_world,
            components: world.components,
            components_array: world.components_array,
            _marker: PhantomData,
        }
    }
}

impl<'a> From<&'a *mut sys::ecs_world_t> for &WorldRef<'a> {
    #[inline(always)]
    fn from(value: &'a *mut sys::ecs_world_t) -> Self {
        unsafe { core::mem::transmute::<&'a *mut sys::ecs_world_t, &WorldRef>(value) }
    }
}

impl Deref for WorldRef<'_> {
    type Target = World;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute::<&WorldRef, &World>(self) }
    }
}

impl<'a> WorldProvider<'a> for WorldRef<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        *self
    }
}

impl<'a> WorldProvider<'a> for &WorldRef<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        **self
    }
}

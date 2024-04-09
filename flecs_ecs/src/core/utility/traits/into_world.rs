use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::core::{Entity, EntityView, Id, Iterable, Query, World, WorldT};

pub trait IntoWorld<'a> {
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        match self.get_world() {
            Some(world) => world.raw_world.as_ptr(),
            None => std::ptr::null_mut(),
        }
    }
    #[inline]
    #[doc(hidden)]
    fn world_ptr(&self) -> *const WorldT {
        match self.get_world() {
            Some(world) => world.raw_world.as_ptr(),
            None => std::ptr::null(),
        }
    }
    #[inline]
    fn world_ref(&self) -> WorldRef<'a> {
        self.get_world()
            .expect("Tried to access world when it was None")
    }
    fn get_world(&self) -> Option<WorldRef<'a>>;
}

impl<'a> IntoWorld<'a> for Id<'a> {
    #[inline]
    fn get_world(&self) -> Option<WorldRef<'a>> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for Entity<'a> {
    #[inline]
    fn get_world(&self) -> Option<WorldRef<'a>> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for EntityView<'a> {
    #[inline]
    fn get_world(&self) -> Option<WorldRef<'a>> {
        self.world
    }
}

impl<'a, T> IntoWorld<'a> for Option<T>
where
    T: IntoWorld<'a>,
{
    #[inline]
    fn get_world(&self) -> Option<WorldRef<'a>> {
        match self {
            Some(s) => s.get_world(),
            None => None,
        }
    }
}

impl<'a, T> IntoWorld<'a> for Query<'a, T>
where
    T: Iterable,
{
    #[inline]
    fn get_world(&self) -> Option<WorldRef<'a>> {
        Some(self.world)
    }
}

impl<'a> IntoWorld<'a> for &'a World {
    #[inline]
    fn get_world(&self) -> Option<WorldRef<'a>> {
        Some((*self).into())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct WorldRef<'a> {
    raw_world: NonNull<WorldT>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> From<&'a World> for WorldRef<'a> {
    fn from(world: &'a World) -> Self {
        WorldRef {
            raw_world: world.raw_world,
            _marker: PhantomData,
        }
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

pub trait FromWorldPtr<'a> {
    /// # Safety
    /// - `raw_world` must be point to a valid world or be null
    unsafe fn from_ptr(raw_world: *mut WorldT) -> Self;
}

impl<'a> FromWorldPtr<'a> for Option<WorldRef<'a>> {
    unsafe fn from_ptr(raw_world: *mut WorldT) -> Self {
        NonNull::new(raw_world).map(|raw_world| WorldRef {
            raw_world,
            _marker: PhantomData,
        })
    }
}

impl<'a> FromWorldPtr<'a> for WorldRef<'a> {
    unsafe fn from_ptr(raw_world: *mut WorldT) -> Self {
        WorldRef {
            raw_world: NonNull::new_unchecked(raw_world),
            _marker: PhantomData,
        }
    }
}

impl<'a> IntoWorld<'a> for WorldRef<'a> {
    fn get_world(&self) -> Option<WorldRef<'a>> {
        Some(*self)
    }
}

impl<'a> IntoWorld<'a> for &WorldRef<'a> {
    fn get_world(&self) -> Option<WorldRef<'a>> {
        Some(**self)
    }
}

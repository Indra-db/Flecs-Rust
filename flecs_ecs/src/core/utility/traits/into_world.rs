use std::{marker::PhantomData, ptr::NonNull};

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
    fn world(&self) -> &'a World {
        self.get_world()
            .expect("Tried to access world when it was None")
    }
    fn get_world(&self) -> Option<&'a World>;
}

impl<'a> IntoWorld<'a> for Id<'a> {
    #[inline]
    fn get_world(&self) -> Option<&'a World> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for Entity<'a> {
    #[inline]
    fn get_world(&self) -> Option<&'a World> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for EntityView<'a> {
    #[inline]
    fn get_world(&self) -> Option<&'a World> {
        self.world
    }
}

impl<'a, T> IntoWorld<'a> for &T
where
    T: IntoWorld<'a>,
{
    #[inline]
    fn get_world(&self) -> Option<&'a World> {
        T::get_world(*self)
    }
}

impl<'a, T> IntoWorld<'a> for &mut T
where
    T: IntoWorld<'a>,
{
    #[inline]
    fn get_world(&self) -> Option<&'a World> {
        T::get_world(*self)
    }
}

impl<'a, T> IntoWorld<'a> for Option<T>
where
    T: IntoWorld<'a>,
{
    #[inline]
    fn get_world(&self) -> Option<&'a World> {
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
    fn get_world(&self) -> Option<&'a World> {
        Some(self.world)
    }
}

impl<'a> IntoWorld<'a> for &'a World {
    #[inline]
    fn get_world(&self) -> Option<&'a World> {
        Some(self)
    }
}

pub struct WorldRef<'a> {
    raw_world: NonNull<WorldT>,
    _marker: PhantomData<&'a ()>,
}

pub trait FromWorldPtr<'a> {
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
    fn get_world(&self) -> Option<&'a World> {
        Some(unsafe { std::mem::transmute::<&WorldRef, &World>(self) })
    }
}

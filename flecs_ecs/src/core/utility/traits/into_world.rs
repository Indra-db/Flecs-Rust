use crate::core::{Entity, EntityView, Id, Iterable, Query, World, WorldT};

pub trait IntoWorld<'a> {
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        unsafe { std::mem::transmute(self.get_world()) }
    }
    #[inline]
    #[doc(hidden)]
    fn world_ptr(&self) -> *const WorldT {
        unsafe { std::mem::transmute(self.get_world()) }
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

pub trait FromWorldPtr<'a> {
    unsafe fn from_ptr(raw_world: *mut WorldT) -> Self;
}

impl<'a> FromWorldPtr<'a> for Option<&'a World> {
    unsafe fn from_ptr(raw_world: *mut WorldT) -> Self {
        if raw_world.is_null() {
            None
        } else {
            Some(std::mem::transmute(raw_world))
        }
    }
}

impl<'a> FromWorldPtr<'a> for &'a World {
    unsafe fn from_ptr(raw_world: *mut WorldT) -> Self {
        std::mem::transmute(raw_world)
    }
}

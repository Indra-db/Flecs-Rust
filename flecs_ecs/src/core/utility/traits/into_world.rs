use crate::core::{Entity, EntityView, Id, Iterable, Query, World, WorldT};

pub trait IntoWorld {
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT;
    #[inline]
    #[doc(hidden)]
    fn get_world_raw(&self) -> *const WorldT {
        self.world_ptr_mut() as *const WorldT
    }
    #[inline]
    fn get_world(&self) -> World {
        World::new_wrap_raw_world(self.world_ptr_mut())
    }
}

impl IntoWorld for *mut WorldT {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        *self
    }
}

impl IntoWorld for *const WorldT {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        *self as *mut WorldT
    }
}

impl IntoWorld for World {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.raw_world
    }
}

impl IntoWorld for Id {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl IntoWorld for Entity {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl IntoWorld for EntityView {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl<T> IntoWorld for &T
where
    T: IntoWorld,
{
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        T::world_ptr_mut(*self)
    }
}

impl<T> IntoWorld for &mut T
where
    T: IntoWorld,
{
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        T::world_ptr_mut(*self)
    }
}

impl<T> IntoWorld for Option<T>
where
    T: IntoWorld,
{
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        match self {
            Some(t) => t.world_ptr_mut(),
            None => std::ptr::null_mut(),
        }
    }
}

impl<T> IntoWorld for Query<T>
where
    T: Iterable,
{
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world.raw_world
    }
}

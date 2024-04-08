use std::marker::PhantomData;

use crate::core::{Entity, EntityView, Id, Iter, Iterable, Query, TermBuilder, World, WorldT};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WorldRef<'a> {
    pub(crate) world: *mut WorldT,
    pub(crate) _marker: PhantomData<&'a World>,
}

impl<'a> WorldRef<'a> {
    pub unsafe fn from_ptr(ptr: *mut WorldT) -> Self {
        Self {
            world: ptr,
            _marker: PhantomData,
        }
    }

    pub fn borrow(&self) -> &'a World {
        unsafe { std::mem::transmute(self.world) }
    }
}

pub trait IntoWorld<'a>: Sized {
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT;

    #[inline]
    #[doc(hidden)]
    fn get_world_raw(&self) -> *const WorldT {
        self.world_ptr_mut() as *const WorldT
    }
    #[inline]
    fn world_ref(&self) -> WorldRef<'a> {
        WorldRef {
            world: self.world_ptr_mut(),
            _marker: PhantomData,
        }
    }
}

impl<'a> IntoWorld<'a> for WorldRef<'a> {
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world
    }
}

// impl IntoWorld<'static> for *mut WorldT {
//     #[inline]
//     #[doc(hidden)]
//     fn world_ptr_mut(&self) -> *mut WorldT {
//         *self
//     }
// }

// impl IntoWorld<'static> for *const WorldT {
//     #[inline]
//     #[doc(hidden)]
//     fn world_ptr_mut(&self) -> *mut WorldT {
//         *self as *mut WorldT
//     }
// }

// impl IntoWorld<'static> for World {
//     #[inline]
//     #[doc(hidden)]
//     fn world_ptr_mut(&self) -> *mut WorldT {
//         self.raw_world
//     }
// }

impl<'a> IntoWorld<'a> for Id<'a> {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world_ptr()
    }
}

impl<'a> IntoWorld<'a> for Entity<'a> {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world_ptr()
    }
}

impl<'a> IntoWorld<'a> for EntityView<'a> {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world_ptr()
    }
}

impl<'a, T: TermBuilder<'a>> IntoWorld<'a> for T {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world_ptr_mut()
    }
}

impl<'a> IntoWorld<'a> for &'a World {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.raw_world
    }
}

impl<'a> IntoWorld<'a> for &'a mut World {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.raw_world
    }
}

impl<'a> IntoWorld<'a> for Iter<'a> {
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.iter.world
    }
}

// impl<'a, T> IntoWorld<'a> for &'a T
// where
//     T: IntoWorld<'a>,
// {
//     #[inline]
//     #[doc(hidden)]
//     fn world_ptr_mut(&self) -> *mut WorldT {
//         T::world_ptr_mut(*self)
//     }
// }

// impl<'a, T> IntoWorld<'a> for &'a mut T
// where
//     T: IntoWorld<'a>,
// {
//     #[inline]
//     #[doc(hidden)]
//     fn world_ptr_mut(&self) -> *mut WorldT {
//         T::world_ptr_mut(*self)
//     }
// }

impl<'a, T> IntoWorld<'a> for Option<T>
where
    T: IntoWorld<'a>,
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

impl<'a, T> IntoWorld<'a> for Query<'a, T>
where
    T: Iterable,
{
    #[inline]
    #[doc(hidden)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world.world_ptr_mut()
    }
}

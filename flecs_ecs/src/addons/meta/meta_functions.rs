use core::ffi::{c_char, c_void};

use crate::core::{Entity, WorldRef};
use flecs_ecs_derive::extern_abi;

use super::{
    AssignBoolFnPtr, AssignCharFnPtr, AssignEntityFnPtr, AssignFloatFnPtr, AssignIntFnPtr,
    AssignNullFnPtr, AssignStringFnPtr, AssignUIntFnPtr, ClearFnPtr, CountFnPtr,
    EnsureElementFnPtr, EnsureMemberFnPtr, ResizeFnPtr, SerializeElementFnPtr, SerializeFnPtr,
    SerializeMemberFnPtr, Serializer,
};

pub trait SerializeFn<T> {
    fn to_extern_fn(self) -> SerializeFnPtr<T>;
}

impl<F, T> SerializeFn<T> for F
where
    F: Fn(&Serializer, &T) -> i32,
{
    fn to_extern_fn(self) -> SerializeFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(ser: &Serializer, value: &T) -> i32
        where
            F: Fn(&Serializer, &T) -> i32,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(ser, value)
        }

        output::<F, T>
    }
}

pub trait SerializeMember<T> {
    fn to_extern_fn(self) -> SerializeMemberFnPtr<T>;
}

impl<F, T> SerializeMember<T> for F
where
    F: Fn(&Serializer, &T, *const c_char) -> i32,
{
    fn to_extern_fn(self) -> SerializeMemberFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(ser: &Serializer, value: &T, name: *const c_char) -> i32
        where
            F: Fn(&Serializer, &T, *const c_char) -> i32,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(ser, value, name)
        }

        output::<F, T>
    }
}

pub trait SerializeElement<T> {
    fn to_extern_fn(self) -> SerializeElementFnPtr<T>;
}
impl<F, T> SerializeElement<T> for F
where
    F: Fn(&Serializer, &T, usize) -> i32,
{
    fn to_extern_fn(self) -> SerializeElementFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(ser: &Serializer, value: &T, elem: usize) -> i32
        where
            F: Fn(&Serializer, &T, usize) -> i32,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(ser, value, elem)
        }

        output::<F, T>
    }
}
pub trait AssignBoolFn<T> {
    fn to_extern_fn(self) -> AssignBoolFnPtr<T>;
}

impl<F, T> AssignBoolFn<T> for F
where
    F: Fn(&mut T, bool),
{
    fn to_extern_fn(self) -> AssignBoolFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: bool)
        where
            F: Fn(&mut T, bool),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignCharFn<T> {
    fn to_extern_fn(self) -> AssignCharFnPtr<T>;
}

impl<F, T> AssignCharFn<T> for F
where
    F: Fn(&mut T, c_char),
{
    fn to_extern_fn(self) -> AssignCharFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: c_char)
        where
            F: Fn(&mut T, c_char),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignIntFn<T> {
    fn to_extern_fn(self) -> AssignIntFnPtr<T>;
}

impl<F, T> AssignIntFn<T> for F
where
    F: Fn(&mut T, i64),
{
    fn to_extern_fn(self) -> AssignIntFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: i64)
        where
            F: Fn(&mut T, i64),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignUIntFn<T> {
    fn to_extern_fn(self) -> AssignUIntFnPtr<T>;
}

impl<F, T> AssignUIntFn<T> for F
where
    F: Fn(&mut T, u64),
{
    fn to_extern_fn(self) -> AssignUIntFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: u64)
        where
            F: Fn(&mut T, u64),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignFloatFn<T> {
    fn to_extern_fn(self) -> AssignFloatFnPtr<T>;
}

impl<F, T> AssignFloatFn<T> for F
where
    F: Fn(&mut T, f32),
{
    fn to_extern_fn(self) -> AssignFloatFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: f32)
        where
            F: Fn(&mut T, f32),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignStringFn<T> {
    fn to_extern_fn(self) -> AssignStringFnPtr<T>;
}

impl<F, T> AssignStringFn<T> for F
where
    F: Fn(&mut T, *const c_char),
{
    fn to_extern_fn(self) -> AssignStringFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: *const c_char)
        where
            F: Fn(&mut T, *const c_char),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignEntityFn<'a, T> {
    fn to_extern_fn(self) -> AssignEntityFnPtr<'a, T>;
}

impl<'a, F, T> AssignEntityFn<'a, T> for F
where
    F: Fn(&mut T, WorldRef<'a>, Entity),
{
    fn to_extern_fn(self) -> AssignEntityFnPtr<'a, T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<'a, F, T>(value: &'a mut T, world: WorldRef<'a>, entity: Entity)
        where
            F: Fn(&'a mut T, WorldRef<'a>, Entity),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, world, entity);
        }

        output::<'a, F, T>
    }
}

pub trait AssignNullFn<T> {
    fn to_extern_fn(self) -> AssignNullFnPtr<T>;
}

impl<F, T> AssignNullFn<T> for F
where
    F: Fn(&mut T),
{
    fn to_extern_fn(self) -> AssignNullFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T)
        where
            F: Fn(&mut T),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value);
        }

        output::<F, T>
    }
}

pub trait ClearFn<T> {
    fn to_extern_fn(self) -> ClearFnPtr<T>;
}

impl<F, T> ClearFn<T> for F
where
    F: Fn(&mut T),
{
    fn to_extern_fn(self) -> ClearFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T)
        where
            F: Fn(&mut T),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value);
        }

        output::<F, T>
    }
}

pub trait EnsureElementFn<T, ELemType> {
    fn to_extern_fn(self) -> EnsureElementFnPtr<T, ELemType>;
}

impl<F, T, ElemType> EnsureElementFn<T, ElemType> for F
where
    F: Fn(&mut T, usize) -> &mut ElemType,
{
    fn to_extern_fn(self) -> EnsureElementFnPtr<T, ElemType> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T, ElemType>(value: &mut T, elem: usize) -> &mut ElemType
        where
            F: Fn(&mut T, usize) -> &mut ElemType,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, elem)
        }

        output::<F, T, ElemType>
    }
}

pub trait EnsureMemberFn<T> {
    fn to_extern_fn(self) -> EnsureMemberFnPtr<T>;
}

impl<F, T> EnsureMemberFn<T> for F
where
    F: Fn(&mut T, *const c_char) -> *mut c_void,
{
    fn to_extern_fn(self) -> EnsureMemberFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: *const c_char) -> *mut c_void
        where
            F: Fn(&mut T, *const c_char) -> *mut c_void,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data)
        }

        output::<F, T>
    }
}

pub trait CountFn<T> {
    fn to_extern_fn(self) -> CountFnPtr<T>;
}

impl<F, T> CountFn<T> for F
where
    F: Fn(&mut T) -> usize,
{
    fn to_extern_fn(self) -> CountFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T) -> usize
        where
            F: Fn(&mut T) -> usize,
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value)
        }

        output::<F, T>
    }
}

pub trait ResizeFn<T> {
    fn to_extern_fn(self) -> ResizeFnPtr<T>;
}

impl<F, T> ResizeFn<T> for F
where
    F: Fn(&mut T, usize),
{
    fn to_extern_fn(self) -> ResizeFnPtr<T> {
        const {
            assert!(core::mem::size_of::<Self>() == 0);
        }
        core::mem::forget(self);

        #[extern_abi]
        fn output<F, T>(value: &mut T, data: usize)
        where
            F: Fn(&mut T, usize),
        {
            (unsafe { core::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

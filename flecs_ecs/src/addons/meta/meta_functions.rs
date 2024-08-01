use std::ffi::c_void;

use crate::core::{Entity, WorldRef};

use super::Serializer;

pub trait SerializeFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&Serializer, &T) -> i32;
}

impl<F, T> SerializeFn<T> for F
where
    F: Fn(&Serializer, &T) -> i32,
{
    fn to_extern_fn(self) -> extern "C" fn(&Serializer, &T) -> i32 {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(ser: &Serializer, value: &T) -> i32
        where
            F: Fn(&Serializer, &T) -> i32,
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(ser, value)
        }

        output::<F, T>
    }
}

pub trait AssignBoolFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, bool);
}

impl<F, T> AssignBoolFn<T> for F
where
    F: Fn(&mut T, bool),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, bool) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: bool)
        where
            F: Fn(&mut T, bool),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignCharFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, i8);
}

impl<F, T> AssignCharFn<T> for F
where
    F: Fn(&mut T, i8),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, i8) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: i8)
        where
            F: Fn(&mut T, i8),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignIntFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, i64);
}

impl<F, T> AssignIntFn<T> for F
where
    F: Fn(&mut T, i64),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, i64) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: i64)
        where
            F: Fn(&mut T, i64),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignUIntFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, u64);
}

impl<F, T> AssignUIntFn<T> for F
where
    F: Fn(&mut T, u64),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, u64) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: u64)
        where
            F: Fn(&mut T, u64),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignFloatFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, f32);
}

impl<F, T> AssignFloatFn<T> for F
where
    F: Fn(&mut T, f32),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, f32) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: f32)
        where
            F: Fn(&mut T, f32),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignStringFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, *const i8);
}

impl<F, T> AssignStringFn<T> for F
where
    F: Fn(&mut T, *const i8),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, *const i8) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: *const i8)
        where
            F: Fn(&mut T, *const i8),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

pub trait AssignEntityFn<'a, T> {
    fn to_extern_fn(self) -> extern "C" fn(&'a mut T, WorldRef<'a>, Entity);
}

impl<'a, F, T> AssignEntityFn<'a, T> for F
where
    F: Fn(&mut T, WorldRef<'a>, Entity),
{
    fn to_extern_fn(self) -> extern "C" fn(&'a mut T, WorldRef<'a>, Entity) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<'a, F, T>(value: &'a mut T, world: WorldRef<'a>, entity: Entity)
        where
            F: Fn(&'a mut T, WorldRef<'a>, Entity),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, world, entity);
        }

        output::<'a, F, T>
    }
}

pub trait AssignNullFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T);
}

impl<F, T> AssignNullFn<T> for F
where
    F: Fn(&mut T),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T)
        where
            F: Fn(&mut T),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value);
        }

        output::<F, T>
    }
}

pub trait ClearFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T);
}

impl<F, T> ClearFn<T> for F
where
    F: Fn(&mut T),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T)
        where
            F: Fn(&mut T),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value);
        }

        output::<F, T>
    }
}

pub trait EnsureElementFn<T, ELemType> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, usize) -> &mut ELemType;
}

impl<F, T, ElemType> EnsureElementFn<T, ElemType> for F
where
    F: Fn(&mut T, usize) -> &mut ElemType,
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, usize) -> &mut ElemType {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T, ElemType>(value: &mut T, elem: usize) -> &mut ElemType
        where
            F: Fn(&mut T, usize) -> &mut ElemType,
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, elem)
        }

        output::<F, T, ElemType>
    }
}

pub trait EnsureMemberFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, *const i8) -> *mut c_void;
}

impl<F, T> EnsureMemberFn<T> for F
where
    F: Fn(&mut T, *const i8) -> *mut c_void,
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, *const i8) -> *mut c_void {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: *const i8) -> *mut c_void
        where
            F: Fn(&mut T, *const i8) -> *mut c_void,
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data)
        }

        output::<F, T>
    }
}

pub trait CountFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T) -> usize;
}

impl<F, T> CountFn<T> for F
where
    F: Fn(&mut T) -> usize,
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T) -> usize {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T) -> usize
        where
            F: Fn(&mut T) -> usize,
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value)
        }

        output::<F, T>
    }
}

pub trait ResizeFn<T> {
    fn to_extern_fn(self) -> extern "C" fn(&mut T, usize);
}

impl<F, T> ResizeFn<T> for F
where
    F: Fn(&mut T, usize),
{
    fn to_extern_fn(self) -> extern "C" fn(&mut T, usize) {
        const {
            assert!(std::mem::size_of::<Self>() == 0);
        }
        std::mem::forget(self);

        extern "C" fn output<F, T>(value: &mut T, data: usize)
        where
            F: Fn(&mut T, usize),
        {
            (unsafe { std::mem::transmute_copy::<_, F>(&()) })(value, data);
        }

        output::<F, T>
    }
}

use crate::core::{
    c_types::{InOutKind, OperKind},
    component_registration::CachedComponentData,
};

use super::traits::{InOutType, OperType};

pub struct Mut<T> {
    phantom: std::marker::PhantomData<T>,
}

pub struct Const<T> {
    phantom: std::marker::PhantomData<T>,
}

pub struct Ref<T> {
    phantom: std::marker::PhantomData<T>,
}

pub struct RefMut<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T> InOutType for Mut<T>
where
    T: CachedComponentData,
{
    const IN_OUT: InOutKind = InOutKind::InOutDefault;
    type Type = T;
}

impl<T> InOutType for Const<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const IN_OUT: InOutKind = InOutKind::In;
}

impl<T> InOutType for Ref<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const IN_OUT: InOutKind = InOutKind::Out;
}

impl<T> InOutType for RefMut<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const IN_OUT: InOutKind = InOutKind::Out;
}

impl<T> InOutType for Option<Const<T>>
where
    T: CachedComponentData,
{
    type Type = T;
    const IN_OUT: InOutKind = InOutKind::In;
}

impl<T> InOutType for Option<Mut<T>>
where
    T: CachedComponentData,
{
    type Type = T;
    const IN_OUT: InOutKind = InOutKind::InOutDefault;
}

impl<T> OperType for Mut<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for Const<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for Ref<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for RefMut<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for Option<T>
where
    T: CachedComponentData,
{
    type Type = T;
    const OPER: OperKind = OperKind::Optional;
}

pub type FTime = f32;

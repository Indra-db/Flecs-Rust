use crate::core::{ComponentId, ComponentType, InOutKind, OperKind, Struct};

use super::IntoComponentId;

/// Represents the input/output type of a component in an ECS system.
///
/// This trait defines the kind of access (input, output, or both) that an ECS system has
/// to a component. Implementing this trait allows specifying whether a component is read,
/// written, or both by a system. This categorization helps in optimizing access patterns
/// and maintaining data consistency within the ECS framework.
///
/// # Associated Constants
///
/// * `IN_OUT`: The kind of access (`InOutKind`) the system has to the component.
///
/// # Associated Types
///
/// * `Type`: The type of the component data. Must implement `ComponentId`.
pub trait InOutType {
    const IN_OUT: InOutKind;
    type Type: IntoComponentId;
}

/// Represents the operation type of a system in an ECS framework.
///
/// This trait is used to specify the kind of operation a system performs on a component,
/// such as adding, removing, or setting a component. Implementing this trait allows the ECS
/// framework to understand and optimize the execution of systems based on their operational
/// characteristics.
///
/// # Associated Constants
///
/// * `OPER`: The kind of operation (`OperKind`) the system performs.
///
/// # Associated Types
///
/// * `Type`: The type of the component data. Must implement `ComponentId`.
pub trait OperType {
    const OPER: OperKind;
    type Type: ComponentId;
}

impl<T> InOutType for &mut T
where
    T: ComponentId,
{
    const IN_OUT: InOutKind = InOutKind::InOutDefault;
    type Type = T;
}

impl<T> InOutType for &T
where
    T: ComponentId,
{
    type Type = T;
    const IN_OUT: InOutKind = InOutKind::In;
}

impl<T, U> InOutType for (T, U)
where
    T: ComponentId,
    U: ComponentId + ComponentType<Struct>,
{
    type Type = (T, U);
    const IN_OUT: InOutKind = InOutKind::InOutDefault;
}

impl<T> OperType for &mut T
where
    T: ComponentId,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for &T
where
    T: ComponentId,
{
    type Type = T;
    const OPER: OperKind = OperKind::And;
}

impl<T> OperType for Option<&T>
where
    T: ComponentId,
{
    type Type = T;
    const OPER: OperKind = OperKind::Optional;
}

impl<T> OperType for Option<&mut T>
where
    T: ComponentId,
{
    type Type = T;
    const OPER: OperKind = OperKind::Optional;
}

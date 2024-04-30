use crate::core::*;

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

use crate::core::{
    c_types::{InOutKind, OperKind},
    component_registration::ComponentInfo,
};

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
/// * `Type`: The type of the component data. Must implement `ComponentInfo`.
pub trait InOutType {
    const IN_OUT: InOutKind;
    type Type: ComponentInfo;
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
/// * `Type`: The type of the component data. Must implement `ComponentInfo`.
pub trait OperType {
    const OPER: OperKind;
    type Type: ComponentInfo;
}

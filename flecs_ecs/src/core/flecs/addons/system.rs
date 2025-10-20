use super::*;
pub type TickSource = crate::sys::EcsTickSource;
impl_component_traits_binding_type_w_id!(TickSource, ECS_TICK_SOURCE);

create_pre_registered_component!(System, ECS_SYSTEM);

create_component_trait!(
    DependsOn,
    ECS_DEPENDS_ON,
    "Builtin relationship. Used to determine the execution order of systems."
);

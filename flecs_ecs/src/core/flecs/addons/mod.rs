use super::*;

#[cfg(feature = "flecs_alerts")]
pub mod alerts;
#[cfg(feature = "flecs_doc")]
pub mod doc;
#[cfg(feature = "flecs_meta")]
pub mod meta;
#[cfg(feature = "flecs_pipeline")]
pub mod pipeline;
#[cfg(feature = "flecs_rest")]
pub mod rest;
#[cfg(feature = "flecs_script")]
pub mod script;
#[cfg(feature = "flecs_system")]
pub mod system;
#[cfg(feature = "flecs_timer")]
pub mod timer;

#[cfg(feature = "flecs_script")]
pub use script::Script;

#[cfg(all(
    not(feature = "flecs_meta"),
    not(feature = "flecs_rust_no_enum_reflection")
))]
pub mod meta {
    use super::*;

    create_pre_registered_component!(I8, ECS_I8_T);
    create_pre_registered_component!(I16, ECS_I16_T);
    create_pre_registered_component!(I32, ECS_I32_T);
    create_pre_registered_component!(I64, ECS_I64_T);
    create_pre_registered_component!(U8, ECS_U8_T);
    create_pre_registered_component!(U16, ECS_U16_T);
    create_pre_registered_component!(U32, ECS_U32_T);
    create_pre_registered_component!(U64, ECS_U64_T);
}

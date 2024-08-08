#[cfg(feature = "flecs_app")]
pub mod app;

#[cfg(feature = "flecs_doc")]
pub mod doc;

#[cfg(feature = "flecs_module")]
pub mod module;

#[cfg(feature = "flecs_system")]
pub mod system;

#[cfg(feature = "flecs_pipeline")]
pub mod pipeline;

#[cfg(feature = "flecs_stats")]
pub mod stats;

#[cfg(feature = "flecs_timer")]
pub mod timer;

#[cfg(feature = "flecs_meta")]
pub mod meta;

#[cfg(feature = "flecs_script")]
pub mod script;

#[cfg(feature = "flecs_json")]
pub mod json;

// this is not feature gated to flecs_meta so calling `.meta()` on a component will always work despite meta being disabled.
pub trait Meta<Component> {
    fn meta(component: flecs_ecs::core::Component<Component>);
}

impl<'a, T: Meta<T>> crate::core::Component<'a, T> {
    pub fn meta(self) -> Self {
        #[cfg(feature = "flecs_meta")]
        {
            T::meta(self);
        }
        self
    }
}

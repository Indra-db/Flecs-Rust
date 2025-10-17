mod module;
pub use module::*;
mod metric_builder;
pub use metric_builder::*;
mod types;
pub use types::*;

mod untyped_component;
mod world;

const ECS_EVENT_DESC_ID_COUNT_MAX: usize = 8;

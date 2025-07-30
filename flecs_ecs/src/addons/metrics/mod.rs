mod module;
pub use module::*;
mod metric_builder;
pub use metric_builder::*;
mod types;
pub use types::*;

use crate::prelude::*;
use crate::sys;

const ECS_EVENT_DESC_ID_COUNT_MAX: usize = 8;

impl<'a> UntypedComponent<'a> {
    /// Register a member as a metric.
    ///
    /// If no explicit name is provided, the metric name will be derived from the member name.
    /// When the member name is `"value"`, the operation will use the name of the component instead.
    ///
    /// If the `brief` parameter is provided, it is set on the metric as if [`set_doc_brief`][crate::addons::doc::Doc::set_doc_brief] was called.
    /// The brief description can be retrieved using [`get_doc_brief`][crate::addons::doc::Doc::doc_brief].
    ///
    /// # Type Parameters
    ///
    /// * `Kind` - The type of the metric (e.g., `Counter`, `CounterIncrement`, or `Gauge`).
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent entity of the metric (optional).
    /// * `brief` - A description for the metric (optional).
    /// * `name` - The name of the metric (optional).
    ///
    /// # See also
    ///
    /// * [`get_doc_brief`][crate::addons::doc::Doc::doc_brief]
    /// * [`set_doc_brief`][crate::addons::doc::Doc::set_doc_brief]
    pub fn metric<Kind>(
        &self,
        parent: Option<impl Into<Entity>>,
        brief: Option<&str>,
        metric_name: Option<&str>,
    ) -> &UntypedComponent<'a>
    where
        Kind: ComponentId + MetricKind,
    {
        let world = self.world();
        let e = EntityView::new_from(world, self.id());

        let member = unsafe { sys::ecs_cpp_last_member(world.world_ptr(), *e.id()) };
        if member.is_null() {
            return self;
        }

        let me = world.entity_from_id(unsafe { (*member).member });
        let mut metric_entity = me;
        if let Some(parent) = parent {
            let component_name = unsafe { e.get_name_cstr() };
            if let Some(metric_name) = metric_name {
                world.run_in_scope_with(parent, || {
                    metric_entity = world.entity_named(metric_name);
                });
            } else {
                let member_name = unsafe { core::ffi::CStr::from_ptr((*member).name) };
                let member_name_str = member_name.to_str().expect("non valid Utf8 conversion");
                if member_name_str == "value" || component_name.is_none() {
                    world.run_in_scope_with(parent, || {
                        metric_entity = world.entity_named(member_name_str);
                    });
                } else {
                    // If name of member is "value", use name of type.
                    if let Some(component_name) = component_name {
                        let snake_name =
                            unsafe { sys::flecs_to_snake_case(component_name.as_ptr()) };
                        let snake_name_str = unsafe {
                            core::ffi::CStr::from_ptr(snake_name)
                                .to_str()
                                .expect("non valid Utf8 conversion")
                        };
                        world.run_in_scope_with(parent, || {
                            metric_entity = world.entity_named(snake_name_str);
                        });
                        unsafe {
                            sys::ecs_os_api.free_.expect("os api is missing")(
                                snake_name as *mut core::ffi::c_void,
                            );
                        };
                    }
                }
            }
        }

        let mut metric = world.metric(metric_entity);
        metric.member(me).kind(Kind::id());
        if let Some(brief) = brief {
            metric.brief(brief);
        }
        self
    }
}

impl World {
    /// Creates a new [`MetricBuilder`] instance.
    ///
    /// # See also
    ///
    /// * [`UntypedComponent::metric()`]
    pub fn metric(&self, entity: impl Into<Entity>) -> MetricBuilder {
        MetricBuilder::new(self, entity.into())
    }
}

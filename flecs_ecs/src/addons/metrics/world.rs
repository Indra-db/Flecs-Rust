use crate::{
    addons::metrics::MetricBuilder,
    core::{Entity, World},
};

impl World {
    /// Creates a new [`MetricBuilder`] instance.
    ///
    /// # See also
    ///
    /// * [`UntypedComponent::metric`](crate::core::UntypedComponent::metric)
    pub fn metric(&self, entity: impl Into<Entity>) -> MetricBuilder<'_> {
        MetricBuilder::new(self, entity.into())
    }
}

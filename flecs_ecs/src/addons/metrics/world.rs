use crate::{addons::metrics::MetricBuilder, core::World};

impl World {
    /// Creates a new [`MetricBuilder`] instance.
    ///
    /// # See also
    ///
    /// * [`UntypedComponent::metric`](crate::core::UntypedComponent::metric)
    pub fn metric(&self, entity: impl crate::core::IntoEntity) -> MetricBuilder<'_> {
        MetricBuilder::new(self, entity.into_entity(self))
    }
}

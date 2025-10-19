
use super::*;

pub type Timer = crate::sys::EcsTimer;
impl_component_traits_binding_type_w_id!(Timer, ECS_TIMER);

pub type RateFilter = crate::sys::EcsRateFilter;
impl_component_traits_binding_type_w_id!(RateFilter, ECS_RATE_FILTER);

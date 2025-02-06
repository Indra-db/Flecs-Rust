use core::ops::Deref;
use core::ptr::addr_of;

use crate::addons::create_pre_registered_extern_component;
use crate::core::*;
use crate::sys;
use flecs_ecs_sys::*; //for all the metrics

pub trait MetricKind {}

create_pre_registered_extern_component!(
    Value,
    FLECS_IDEcsMetricValueID_,
    "Component with metric instance value."
);

create_pre_registered_extern_component!(
    MetricInstance,
    EcsMetricInstance,
    "Tag added to metric instances."
);
create_pre_registered_extern_component!(
    Metric,
    EcsMetric,
    "Tag added to metrics, and used as first element of metric kind pair."
);
create_pre_registered_extern_component!(
    Counter,
    EcsCounter,
    "Metric that has monotonically increasing value."
);
create_pre_registered_extern_component!(
    CounterIncrement,
    EcsCounterIncrement,
    "Counter metric that is auto-incremented by source value."
);
create_pre_registered_extern_component!(
    CounterId,
    EcsCounterId,
    "Counter metric that counts the number of entities with an id."
);
create_pre_registered_extern_component!(Gauge, EcsGauge, "Metric that represents current value.");

impl MetricKind for Value {}
impl MetricKind for MetricInstance {}
impl MetricKind for Metric {}
impl MetricKind for Counter {}
impl MetricKind for CounterIncrement {}
impl MetricKind for CounterId {}
impl MetricKind for Gauge {}

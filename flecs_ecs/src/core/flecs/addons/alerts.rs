//! Alerts pre-registered components.

use super::*;
use crate::addons::create_pre_registered_extern_component;
use crate::sys::*;

create_pre_registered_extern_component!(
    AlertComponent,
    FLECS_IDEcsAlertID_,
    "Component added to alert, and used as first element of alert severity pair."
);

create_pre_registered_extern_component!(Info, EcsAlertInfo, "Info alert severity.");
create_pre_registered_extern_component!(Warning, EcsAlertWarning, "Warning alert severity.");
create_pre_registered_extern_component!(Error, EcsAlertError, "Error alert severity.");
create_pre_registered_extern_component!(Critical, EcsAlertCritical, "Critical alert severity.");

create_pre_registered_extern_component!(
    AlertsActive,
    FLECS_IDEcsAlertsActiveID_,
    "Component added to alert source which tracks how many active alerts there are."
);
create_pre_registered_extern_component!(
    AlertInstance,
    FLECS_IDEcsAlertInstanceID_,
    "Component added to alert instance."
);
create_pre_registered_extern_component!(
    AlertTimeout,
    FLECS_IDEcsAlertTimeoutID_,
    "Component added to alert which tracks how long an alert has been inactive."
);

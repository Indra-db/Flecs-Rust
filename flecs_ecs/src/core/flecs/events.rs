use super::*;

create_pre_registered_component!(
    OnAdd,
    ECS_ON_ADD,
    "Event. Invoked whenever a component, tag or pair is added to an entity."
);
create_pre_registered_component!(
    OnRemove,
    ECS_ON_REMOVE,
    "Event. Invoked whenever a component, tag or pair is removed from an entity."
);
create_pre_registered_component!(
    OnSet,
    ECS_ON_SET,
    "Event. Invoked whenever a component is assigned a new value."
);
create_pre_registered_component!(OnTableCreate, ECS_ON_TABLE_CREATE);
create_pre_registered_component!(OnTableDelete, ECS_ON_TABLE_DELETE);

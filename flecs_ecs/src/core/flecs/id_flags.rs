//! Contains flags, components used for ids

use super::*;
create_pre_registered_component!(Pair, ECS_PAIR, "Indicates that the id is a pair.");
create_pre_registered_component!(
    AutoOverride,
    ECS_AUTO_OVERRIDE,
    "Automatically override component when it is inherited"
);
create_pre_registered_component!(
    Toggle,
    ECS_TOGGLE,
    "Adds bitset to storage which allows component to be enabled/disabled"
);
create_pre_registered_component!(
    And,
    ECS_AND,
    "Include all components from entity to which AND is applied"
);

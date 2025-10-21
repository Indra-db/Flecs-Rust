//! Contains phases of pipeline

use super::*;
create_pre_registered_component!(Pipeline, ECS_PIPELINE);
create_pre_registered_component!(OnStart, ECS_ON_START);
//create_pre_registered_component!(PreFrame, ECS_PRE_FRAME); //not meant to be exposed, internal only
create_pre_registered_component!(OnLoad, ECS_ON_LOAD);
create_pre_registered_component!(PostLoad, ECS_POST_LOAD);
create_pre_registered_component!(PreUpdate, ECS_PRE_UPDATE);
create_pre_registered_component!(OnUpdate, ECS_ON_UPDATE);
create_pre_registered_component!(OnValidate, ECS_ON_VALIDATE);
create_pre_registered_component!(PostUpdate, ECS_POST_UPDATE);
create_pre_registered_component!(PreStore, ECS_PRE_STORE);
create_pre_registered_component!(OnStore, ECS_ON_STORE);
//create_pre_registered_component!(PostFrame, ECS_POST_FRAME); //not meant to be exposed, internal only
create_pre_registered_component!(Phase, ECS_PHASE);

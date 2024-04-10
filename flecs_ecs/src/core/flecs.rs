use std::{ffi::CStr, sync::OnceLock};

use crate::core::*;

#[macro_export]
macro_rules! create_pre_registered_component {
    ($struct_name:ident, $const_name:ident) => {
        pub struct $struct_name;

        impl FlecsConstantId for $struct_name {
            const ID: u64 = $const_name;
        }

        impl ComponentInfo for $struct_name {
            const IS_ENUM: bool = false;
            const IS_TAG: bool = true;
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = false;
        }

        impl EmptyComponent for $struct_name {}

        impl ComponentType<Struct> for $struct_name {}

        impl ComponentId for $struct_name {
            type UnderlyingType = $struct_name;
            type UnderlyingEnumType = NoneEnum;

            fn register_explicit<'a>(_world: impl IntoWorld<'a>) {}

            fn register_explicit_named<'a>(_world: impl IntoWorld<'a>, _name: &CStr) -> EntityT {
                $const_name
            }

            fn is_registered() -> bool {
                true
            }

            fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
                true
            }

            fn get_id<'a>(_world: impl IntoWorld<'a>) -> IdT {
                $const_name
            }

            unsafe fn get_id_unchecked() -> IdT {
                $const_name
            }

            fn __get_once_lock_data() -> &'static OnceLock<IdComponent> {
                static ONCE_LOCK: OnceLock<IdComponent> = OnceLock::new();
                &ONCE_LOCK
            }
        }
    };
}

// Indicates that the id is a pair.
create_pre_registered_component!(Pair, ECS_PAIR);
// Automatically override component when it is inherited
create_pre_registered_component!(Override, ECS_OVERRIDE);
// Adds bitset to storage which allows component to be enabled/disabled
create_pre_registered_component!(Toggle, ECS_TOGGLE);
// Include all components from entity to which AND is applied
create_pre_registered_component!(And, ECS_AND);

// Builtin component ids
create_pre_registered_component!(Component, ECS_COMPONENT);
create_pre_registered_component!(FieldIdentifier, ecs_field_idENTIFIER);
create_pre_registered_component!(Iterable, ECS_ITERABLE);
create_pre_registered_component!(Poly, ECS_POLY);

// Poly target components
create_pre_registered_component!(Query, ECS_QUERY);
create_pre_registered_component!(Observer, ECS_OBSERVER);

// Core scopes & entities
create_pre_registered_component!(World, ECS_WORLD);
create_pre_registered_component!(Flecs, ECS_FLECS);
create_pre_registered_component!(FlecsCore, ECS_FLECS_CORE);
create_pre_registered_component!(FlecsInternals, ECS_FLECS_INTERNALS);
create_pre_registered_component!(Module, ECS_MODULE);
create_pre_registered_component!(Private, ECS_PRIVATE);
create_pre_registered_component!(Prefab, ECS_PREFAB);
create_pre_registered_component!(Disabled, ECS_DISABLED);
create_pre_registered_component!(SlotOf, ECS_SLOT_OF);
create_pre_registered_component!(Flag, ECS_FLAG);
create_pre_registered_component!(Monitor, ECS_MONITOR);
create_pre_registered_component!(Empty, ECS_EMPTY);

// Relationship properties
create_pre_registered_component!(Wildcard, ECS_WILDCARD);
create_pre_registered_component!(Any, ECS_ANY);
create_pre_registered_component!(This_, ECS_THIS);
create_pre_registered_component!(Variable, ECS_VARIABLE);
create_pre_registered_component!(Transitive, ECS_TRANSITIVE);
create_pre_registered_component!(Reflexive, ECS_REFLEXIVE);
create_pre_registered_component!(Symmetric, ECS_SYMMETRIC);
create_pre_registered_component!(Final, ECS_FINAL);
create_pre_registered_component!(DontInherit, ECS_DONT_INHERIT);
create_pre_registered_component!(AlwaysOverride, ECS_ALWAYS_OVERRIDE);
create_pre_registered_component!(Tag, ECS_TAG);
create_pre_registered_component!(Union, ECS_UNION);
create_pre_registered_component!(Exclusive, ECS_EXCLUSIVE);
create_pre_registered_component!(Acyclic, ECS_ACYCLIC);
create_pre_registered_component!(Traversable, ECS_TRAVERSABLE);
create_pre_registered_component!(With, ECS_WITH);
create_pre_registered_component!(OneOf, ECS_ONE_OF);

// Builtin relationships
create_pre_registered_component!(ChildOf, ECS_CHILD_OF);
create_pre_registered_component!(IsA, ECS_IS_A);
create_pre_registered_component!(DependsOn, ECS_DEPENDS_ON);

// Identifier tags
create_pre_registered_component!(Name, ECS_NAME);
create_pre_registered_component!(Symbol, ECS_SYMBOL);
create_pre_registered_component!(Alias, ECS_ALIAS);

// Events

create_pre_registered_component!(OnAdd, ECS_ON_ADD);
create_pre_registered_component!(OnRemove, ECS_ON_REMOVE);
create_pre_registered_component!(OnSet, ECS_ON_SET);
create_pre_registered_component!(Unset, ECS_UNSET);
create_pre_registered_component!(OnDelete, ECS_ON_DELETE);
create_pre_registered_component!(OnTableCreate, ECS_ON_TABLE_CREATE);
create_pre_registered_component!(OnTableDelete, ECS_ON_TABLE_DELETE);
create_pre_registered_component!(OnTableEmpty, ECS_ON_TABLE_EMPTY);
create_pre_registered_component!(OnTableFill, ECS_ON_TABLE_FILL);
create_pre_registered_component!(OnCreateTrigger, ECS_ON_CREATE_TRIGGER);
create_pre_registered_component!(OnDeleteTrigger, ECS_ON_DELETE_TRIGGER);
create_pre_registered_component!(OnDeleteObservable, ECS_ON_DELETE_OBSERVABLE);
create_pre_registered_component!(OnComponentHooks, ECS_ON_COMPONENT_HOOKS);
create_pre_registered_component!(OnDeleteTarget, ECS_ON_DELETE_TARGET);

// System
pub mod system {
    use super::*;
    create_pre_registered_component!(System, ECS_SYSTEM);
    create_pre_registered_component!(TickSource, ECS_TICK_SOURCE);
}

pub mod timers {
    use super::*;
    create_pre_registered_component!(Timer, ECS_TIMER);
    create_pre_registered_component!(RateFilter, ECS_RATE_FILTER);
}

// Actions
create_pre_registered_component!(Remove, ECS_REMOVE);
create_pre_registered_component!(Delete, ECS_DELETE);
create_pre_registered_component!(Panic, ECS_PANIC);

// Misc
create_pre_registered_component!(Target, ECS_TARGET);
create_pre_registered_component!(Flatten, ECS_FLATTEN);
create_pre_registered_component!(DefaultChildComponent, ECS_DEFAULT_CHILD_COMPONENT);

// Builtin predicate ids (used by rule engine)
create_pre_registered_component!(PredEq, ECS_PRED_EQ);
create_pre_registered_component!(PredMatch, ECS_PRED_MATCH);
create_pre_registered_component!(PredLookup, ECS_PRED_LOOKUP);
create_pre_registered_component!(ScopeOpen, ECS_SCOPE_OPEN);
create_pre_registered_component!(ScopeClose, ECS_SCOPE_CLOSE);

// Systems
pub mod pipeline {
    use super::*;
    create_pre_registered_component!(Pipeline, ECS_PIPELINE);
    create_pre_registered_component!(OnStart, ECS_ON_START);
    create_pre_registered_component!(PreFrame, ECS_PRE_FRAME);
    create_pre_registered_component!(OnLoad, ECS_ON_LOAD);
    create_pre_registered_component!(PostLoad, ECS_POST_LOAD);
    create_pre_registered_component!(PreUpdate, ECS_PRE_UPDATE);
    create_pre_registered_component!(OnUpdate, ECS_ON_UPDATE);
    create_pre_registered_component!(OnValidate, ECS_ON_VALIDATE);
    create_pre_registered_component!(PostUpdate, ECS_POST_UPDATE);
    create_pre_registered_component!(PreStore, ECS_PRE_STORE);
    create_pre_registered_component!(OnStore, ECS_ON_STORE);
    create_pre_registered_component!(PostFrame, ECS_POST_FRAME);
    create_pre_registered_component!(Phase, ECS_PHASE);
}

pub mod meta {
    use super::*;
    // Meta primitive components (don't use low ids to save id space)
    create_pre_registered_component!(Bool, ECS_BOOL_T);
    create_pre_registered_component!(Char, ECS_CHAR_T);
    create_pre_registered_component!(Byte, ECS_BYTE_T);
    create_pre_registered_component!(U8, ECS_U8_T);
    create_pre_registered_component!(U16, ECS_U16_T);
    create_pre_registered_component!(U32, ECS_U32_T);
    create_pre_registered_component!(U64, ECS_U64_T);
    create_pre_registered_component!(UPtr, ECS_UPTR_T);
    create_pre_registered_component!(I8, ECS_I8_T);
    create_pre_registered_component!(I16, ECS_I16_T);
    create_pre_registered_component!(I32, ECS_I32_T);
    create_pre_registered_component!(I64, ECS_I64_T);
    create_pre_registered_component!(IPtr, ECS_IPTR_T);
    create_pre_registered_component!(F32, ECS_F32_T);
    create_pre_registered_component!(F64, ECS_F64_T);
    create_pre_registered_component!(String, ECS_STRING_T);
    create_pre_registered_component!(Entity, ECS_ENTITY_T);

    // Meta type components
    create_pre_registered_component!(Type, ECS_META_TYPE);
    create_pre_registered_component!(TypeSerialized, ECS_META_TYPE_SERIALIZED);
    create_pre_registered_component!(Primitive, ECS_PRIMITIVE);
    create_pre_registered_component!(Enum, ECS_ENUM);
    create_pre_registered_component!(Bitmask, ECS_BITMASK);
    create_pre_registered_component!(Member, ECS_MEMBER);
    create_pre_registered_component!(StructT, ECS_STRUCT);
    create_pre_registered_component!(Array, ECS_ARRAY);
    create_pre_registered_component!(Vector, ECS_VECTOR);
    create_pre_registered_component!(Opaque, ECS_OPAQUE);
    create_pre_registered_component!(Unit, ECS_UNIT);
    create_pre_registered_component!(UnitPrefix, ECS_UNIT_PREFIX);
    create_pre_registered_component!(Constant, ECS_CONSTANT);
    create_pre_registered_component!(Quantity, ECS_QUANTITY);
}

// Doc module components
pub mod doc {
    use super::*;
    create_pre_registered_component!(Description, ECS_DOC_DESCRIPTION);
    create_pre_registered_component!(Brief, ECS_DOC_BRIEF);
    create_pre_registered_component!(Detail, ECS_DOC_DETAIL);
    create_pre_registered_component!(Link, ECS_DOC_LINK);
    create_pre_registered_component!(Color, ECS_DOC_COLOR);
}

pub mod rest {
    use super::*;
    // REST module components
    create_pre_registered_component!(Rest, ECS_REST);
}

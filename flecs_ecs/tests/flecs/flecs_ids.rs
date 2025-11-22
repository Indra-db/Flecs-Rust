use flecs_ecs::prelude::*;
use flecs_ecs::sys;

#[test]
fn test_c_vs_rust_ids() {
    let world = flecs_ecs::core::World::new();

    unsafe {
        assert_eq!(
            flecs::term_flags::Self_,
            sys::EcsSelf as u64,
            "EcsSelf (C) != Self_ (Rust)"
        );
        assert_eq!(flecs::term_flags::Up, sys::EcsUp, "EcsUp (C) != Up (Rust)");
        assert_eq!(
            flecs::term_flags::Trav,
            sys::EcsTrav,
            "EcsTrav (C) != Trav (Rust)"
        );
        assert_eq!(
            flecs::term_flags::Cascade,
            sys::EcsCascade,
            "EcsCascade (C) != Cascade (Rust)"
        );
        assert_eq!(
            flecs::term_flags::Desc,
            sys::EcsDesc,
            "EcsDesc (C) != Desc (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsVariable,
            sys::EcsIsVariable,
            "EcsIsVariable (C) != IsVariable (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsEntity,
            sys::EcsIsEntity,
            "EcsIsEntity (C) != IsEntity (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsName,
            sys::EcsIsName,
            "EcsIsName (C) != IsName (Rust)"
        );
        assert_eq!(
            flecs::term_flags::TraverseFlags,
            sys::EcsTraverseFlags as u64,
            "EcsTraverseFlags (C) != TraverseFlags (Rust)"
        );
        assert_eq!(
            flecs::term_flags::TermRefFlags,
            sys::EcsTermRefFlags as u64,
            "EcsTermRefFlags (C) != TermRefFlags (Rust)"
        );

        // Term flags
        assert_eq!(
            flecs::term_flags::MatchAny,
            sys::EcsTermMatchAny as u64,
            "EcsTermMatchAny (C) != MatchAny (Rust)"
        );
        assert_eq!(
            flecs::term_flags::MatchAnySrc,
            sys::EcsTermMatchAnySrc as u64,
            "EcsTermMatchAnySrc (C) != MatchAnySrc (Rust)"
        );
        assert_eq!(
            flecs::term_flags::Transitive,
            sys::EcsTermTransitive as u64,
            "EcsTermTransitive (C) != Transitive (Rust)"
        );
        assert_eq!(
            flecs::term_flags::Reflexive,
            sys::EcsTermReflexive as u64,
            "EcsTermReflexive (C) != Reflexive (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IdInherited,
            sys::EcsTermIdInherited as u64,
            "EcsTermIdInherited (C) != IdInherited (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsTrivial,
            sys::EcsTermIsTrivial as u64,
            "EcsTermIsTrivial (C) != IsTrivial (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsCacheable,
            sys::EcsTermIsCacheable as u64,
            "EcsTermIsCacheable (C) != IsCacheable (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsScope,
            sys::EcsTermIsScope as u64,
            "EcsTermIsScope (C) != IsScope (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsMember,
            sys::EcsTermIsMember as u64,
            "EcsTermIsMember (C) != IsMember (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsToggle,
            sys::EcsTermIsToggle as u64,
            "EcsTermIsToggle (C) != IsToggle (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsSparse,
            sys::EcsTermIsSparse as u64,
            "EcsTermIsSparse (C) != IsSparse (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsOr,
            sys::EcsTermIsOr as u64,
            "EcsTermIsOr (C) != IsOr (Rust)"
        );
        assert_eq!(
            flecs::term_flags::IsDontFragment,
            sys::EcsTermDontFragment as u64,
            "EcsTermDontFragment (C) != IsDontFragment (Rust)"
        );

        // Query flags
        assert_eq!(
            flecs::query_flags::MatchPrefab,
            sys::EcsQueryMatchPrefab as u64,
            "EcsQueryMatchPrefab (C) != MatchPrefab (Rust)"
        );
        assert_eq!(
            flecs::query_flags::MatchDisabled,
            sys::EcsQueryMatchDisabled as u64,
            "EcsQueryMatchDisabled (C) != MatchDisabled (Rust)"
        );
        assert_eq!(
            flecs::query_flags::MatchEmptyTables,
            sys::EcsQueryMatchEmptyTables as u64,
            "EcsQueryMatchEmptyTables (C) != MatchEmptyTables (Rust)"
        );
        assert_eq!(
            flecs::query_flags::AllowUnresolvedByName,
            sys::EcsQueryAllowUnresolvedByName as u64,
            "EcsQueryAllowUnresolvedByName (C) != AllowUnresolvedByName (Rust)"
        );
        assert_eq!(
            flecs::query_flags::TableOnly,
            sys::EcsQueryTableOnly as u64,
            "EcsQueryTableOnly (C) != TableOnly (Rust)"
        );

        assert_eq!(flecs::Component::ID, sys::FLECS_IDEcsComponentID_);
        assert_eq!(flecs::Identifier::ID, sys::FLECS_IDEcsIdentifierID_);
        assert_eq!(flecs::Poly::ID, sys::FLECS_IDEcsPolyID_);
        assert_eq!(
            flecs::DefaultChildComponent::ID,
            sys::FLECS_IDEcsDefaultChildComponentID_
        );

        // Poly target components
        assert_eq!(flecs::Query, sys::EcsQuery);
        assert_eq!(flecs::Observer, sys::EcsObserver);

        // Core scopes & entities
        assert_eq!(flecs::EcsWorld, sys::EcsWorld);
        assert_eq!(flecs::Flecs, sys::EcsFlecs);
        assert_eq!(flecs::FlecsCore, sys::EcsFlecsCore);
        //assert_eq!(flecs::FlecsInternals, sys::EcsFlecsInternals);
        assert_eq!(flecs::Module, sys::EcsModule);
        assert_eq!(flecs::Private, sys::EcsPrivate);
        assert_eq!(flecs::Prefab, sys::EcsPrefab);
        assert_eq!(flecs::Disabled, sys::EcsDisabled);
        assert_eq!(flecs::NotQueryable, sys::EcsNotQueryable);
        assert_eq!(flecs::SlotOf, sys::EcsSlotOf);
        assert_eq!(flecs::OrderedChildren, sys::EcsOrderedChildren);
        //assert_eq!(flecs::Flag, sys::EcsFlag);
        assert_eq!(flecs::Monitor, sys::EcsMonitor);
        assert_eq!(flecs::Empty, sys::EcsEmpty);
        assert_eq!(flecs::Constant, sys::EcsConstant);

        // Component traits
        assert_eq!(flecs::Wildcard, sys::EcsWildcard);
        assert_eq!(flecs::Any, sys::EcsAny);
        assert_eq!(flecs::This_, sys::EcsThis);
        assert_eq!(flecs::Variable, sys::EcsVariable);
        assert_eq!(flecs::Singleton, sys::EcsSingleton);
        assert_eq!(flecs::Transitive, sys::EcsTransitive);
        assert_eq!(flecs::Reflexive, sys::EcsReflexive);
        assert_eq!(flecs::Symmetric, sys::EcsSymmetric);
        assert_eq!(flecs::Final, sys::EcsFinal);
        assert_eq!(flecs::Inheritable, sys::EcsInheritable);
        assert_eq!(flecs::PairIsTag, sys::EcsPairIsTag);
        assert_eq!(flecs::Exclusive, sys::EcsExclusive);
        assert_eq!(flecs::Acyclic, sys::EcsAcyclic);
        assert_eq!(flecs::Traversable, sys::EcsTraversable);
        assert_eq!(flecs::With, sys::EcsWith);
        assert_eq!(flecs::OneOf, sys::EcsOneOf);
        assert_eq!(flecs::CanToggle, sys::EcsCanToggle);
        assert_eq!(flecs::Trait, sys::EcsTrait);
        assert_eq!(flecs::Relationship, sys::EcsRelationship);
        assert_eq!(flecs::Target, sys::EcsTarget);

        // OnInstantiate traits
        assert_eq!(flecs::OnInstantiate, sys::EcsOnInstantiate);
        assert_eq!(flecs::Override, sys::EcsOverride);
        assert_eq!(flecs::Inherit, sys::EcsInherit);
        assert_eq!(flecs::DontInherit, sys::EcsDontInherit);

        // OnDelete/OnDeleteTarget traits
        assert_eq!(flecs::OnDelete, sys::EcsOnDelete);
        assert_eq!(flecs::OnDeleteTarget, sys::EcsOnDeleteTarget);
        assert_eq!(flecs::Remove, sys::EcsRemove);
        assert_eq!(flecs::Delete, sys::EcsDelete);
        assert_eq!(flecs::Panic, sys::EcsPanic);

        // Builtin relationships
        assert_eq!(flecs::ChildOf, sys::EcsChildOf);
        assert_eq!(flecs::IsA, sys::EcsIsA);
        assert_eq!(flecs::DependsOn, sys::EcsDependsOn);

        // Identifier tags
        assert_eq!(flecs::Name, sys::EcsName);
        assert_eq!(flecs::Symbol, sys::EcsSymbol);
        assert_eq!(flecs::Alias, sys::EcsAlias);

        // Events
        assert_eq!(flecs::OnAdd, sys::EcsOnAdd);
        assert_eq!(flecs::OnRemove, sys::EcsOnRemove);
        assert_eq!(flecs::OnSet, sys::EcsOnSet);
        assert_eq!(flecs::OnTableCreate, sys::EcsOnTableCreate);
        assert_eq!(flecs::OnTableDelete, sys::EcsOnTableDelete);

        // System
        #[cfg(feature = "flecs_system")]
        {
            assert_eq!(flecs::system::TickSource::ID, sys::FLECS_IDEcsTickSourceID_);
            assert_eq!(flecs::system::System, sys::EcsSystem);
        }

        // Timer
        #[cfg(feature = "flecs_timer")]
        {
            assert_eq!(flecs::timer::Timer::ID, sys::FLECS_IDEcsTimerID_);
            assert_eq!(flecs::timer::RateFilter::ID, sys::FLECS_IDEcsRateFilterID_);
        }

        // Script
        #[allow(static_mut_refs)]
        #[cfg(feature = "flecs_script")]
        {
            assert_eq!(
                flecs::script::Script::__register_or_get_id::<false>(&world),
                sys::FLECS_IDEcsScriptID_
            );
        }

        assert_eq!(
            flecs::Sparse,
            sys::EcsSparse,
            "EcsSparse (C) != Sparse (Rust)",
        );
        assert_eq!(
            flecs::DontFragment,
            sys::EcsDontFragment,
            "EcsDontFragment (C) != DontFragment (Rust)",
        );

        // Builtin predicate for comparing entity ids
        assert_eq!(flecs::PredEq, sys::EcsPredEq);
        assert_eq!(flecs::PredMatch, sys::EcsPredMatch);
        assert_eq!(flecs::PredLookup, sys::EcsPredLookup);

        // builtin marker entities for query scopes
        assert_eq!(flecs::ScopeOpen, sys::EcsScopeOpen);
        assert_eq!(flecs::ScopeClose, sys::EcsScopeClose);

        // Pipeline
        #[cfg(feature = "flecs_pipeline")]
        {
            assert_eq!(flecs::pipeline::Pipeline, sys::FLECS_IDEcsPipelineID_);
            assert_eq!(flecs::pipeline::OnStart, sys::EcsOnStart);
            assert_eq!(flecs::pipeline::OnLoad, sys::EcsOnLoad);
            assert_eq!(flecs::pipeline::PostLoad, sys::EcsPostLoad);
            assert_eq!(flecs::pipeline::PreUpdate, sys::EcsPreUpdate);
            assert_eq!(flecs::pipeline::OnUpdate, sys::EcsOnUpdate);
            assert_eq!(flecs::pipeline::OnValidate, sys::EcsOnValidate);
            assert_eq!(flecs::pipeline::PostUpdate, sys::EcsPostUpdate);
            assert_eq!(flecs::pipeline::PreStore, sys::EcsPreStore);
            assert_eq!(flecs::pipeline::OnStore, sys::EcsOnStore);
            assert_eq!(flecs::pipeline::Phase, sys::EcsPhase);
        }

        // Meta
        #[cfg(feature = "flecs_meta")]
        {
            assert_eq!(flecs::meta::Bool, sys::FLECS_IDecs_bool_tID_);
            assert_eq!(flecs::meta::Char, sys::FLECS_IDecs_char_tID_);
            assert_eq!(flecs::meta::Byte, sys::FLECS_IDecs_byte_tID_);
            assert_eq!(flecs::meta::U8, sys::FLECS_IDecs_u8_tID_);
            assert_eq!(flecs::meta::U16, sys::FLECS_IDecs_u16_tID_);
            assert_eq!(flecs::meta::U32, sys::FLECS_IDecs_u32_tID_);
            assert_eq!(flecs::meta::U64, sys::FLECS_IDecs_u64_tID_);
            assert_eq!(flecs::meta::UPtr, sys::FLECS_IDecs_uptr_tID_);
            assert_eq!(flecs::meta::I8, sys::FLECS_IDecs_i8_tID_);
            assert_eq!(flecs::meta::I16, sys::FLECS_IDecs_i16_tID_);
            assert_eq!(flecs::meta::I32, sys::FLECS_IDecs_i32_tID_);
            assert_eq!(flecs::meta::I64, sys::FLECS_IDecs_i64_tID_);
            assert_eq!(flecs::meta::IPtr, sys::FLECS_IDecs_iptr_tID_);
            assert_eq!(flecs::meta::F32, sys::FLECS_IDecs_f32_tID_);
            assert_eq!(flecs::meta::F64, sys::FLECS_IDecs_f64_tID_);
            assert_eq!(flecs::meta::String, sys::FLECS_IDecs_string_tID_);
            assert_eq!(flecs::meta::Entity, sys::FLECS_IDecs_entity_tID_);
            assert_eq!(flecs::meta::Id, sys::FLECS_IDecs_id_tID_);
            assert_eq!(flecs::meta::Quantity, sys::EcsQuantity);
            assert_eq!(flecs::meta::EcsOpaque, sys::FLECS_IDEcsOpaqueID_);

            assert_eq!(flecs::meta::Type::ID, sys::FLECS_IDEcsTypeID_);
            assert_eq!(
                flecs::meta::TypeSerializer::ID,
                sys::FLECS_IDEcsTypeSerializerID_
            );
            assert_eq!(flecs::meta::Primitive::ID, sys::FLECS_IDEcsPrimitiveID_);
            assert_eq!(flecs::meta::EcsEnum::ID, sys::FLECS_IDEcsEnumID_);
            assert_eq!(flecs::meta::Bitmask::ID, sys::FLECS_IDEcsBitmaskID_);
            assert_eq!(flecs::meta::Member::ID, sys::FLECS_IDEcsMemberID_);
            assert_eq!(
                flecs::meta::MemberRanges::ID,
                sys::FLECS_IDEcsMemberRangesID_
            );
            assert_eq!(flecs::meta::EcsStruct::ID, sys::FLECS_IDEcsStructID_);
            assert_eq!(flecs::meta::Array::ID, sys::FLECS_IDEcsArrayID_);
            assert_eq!(flecs::meta::Vector::ID, sys::FLECS_IDEcsVectorID_);
            assert_eq!(flecs::meta::Unit::ID, sys::FLECS_IDEcsUnitID_);
            assert_eq!(flecs::meta::UnitPrefix::ID, sys::FLECS_IDEcsUnitPrefixID_);
        }

        // Doc
        #[cfg(feature = "flecs_doc")]
        {
            assert_eq!(flecs::doc::Description, sys::FLECS_IDEcsDocDescriptionID_);
            assert_eq!(flecs::doc::Brief, sys::EcsDocBrief);
            assert_eq!(flecs::doc::Detail, sys::EcsDocDetail);
            assert_eq!(flecs::doc::Link, sys::EcsDocLink);
            assert_eq!(flecs::doc::Color, sys::EcsDocColor);
            assert_eq!(flecs::doc::UUID, sys::EcsDocUuid);
        }

        // Rest
        #[cfg(feature = "flecs_rest")]
        {
            assert_eq!(flecs::rest::Rest::ID, sys::FLECS_IDEcsRestID_);
        }
    }
}

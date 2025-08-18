use crate::sys;

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct TableFlags: u32 {
        const HasBuiltins = sys::EcsTableHasBuiltins;
        const IsPrefab = sys::EcsTableIsPrefab;
        const HasIsA = sys::EcsTableHasIsA;
        const HasChildOf = sys::EcsTableHasChildOf;
        const HasName = sys::EcsTableHasName;
        const HasPairs = sys::EcsTableHasPairs;
        const HasModule = sys::EcsTableHasModule;
        const IsDisabled = sys::EcsTableIsDisabled;
        const NotQueryable = sys::EcsTableNotQueryable;
        const HasCtors = sys::EcsTableHasCtors;
        const HasDtors = sys::EcsTableHasDtors;
        const HasCopy = sys::EcsTableHasCopy;
        const HasMove = sys::EcsTableHasMove;
        const HasToggle = sys::EcsTableHasToggle;
        const HasOverrides = sys::EcsTableHasOverrides;
        const HasOnAdd = sys::EcsTableHasOnAdd;
        const HasOnRemove = sys::EcsTableHasOnRemove;
        const HasOnSet = sys::EcsTableHasOnSet;
        const HasOnTableCreate = sys::EcsTableHasOnTableCreate;
        const HasOnTableDelete = sys::EcsTableHasOnTableDelete;
        const HasSparse = sys::EcsTableHasSparse;
        const HasDontFragment = sys::EcsTableHasDontFragment;
        const OverrideDontFragment = sys::EcsTableOverrideDontFragment;
        const HasTraversable = sys::EcsTableHasTraversable;
        const HasOrderedChildren = sys::EcsTableHasOrderedChildren;
        const EdgeReparent = sys::EcsTableEdgeReparent;
        const MarkedForDelete = sys::EcsTableMarkedForDelete;
        const HasLifecycle = sys::EcsTableHasLifecycle;
        const IsComplex = sys::EcsTableIsComplex;
        const HasAddActions = sys::EcsTableHasAddActions;
        const HasRemoveActions = sys::EcsTableHasRemoveActions;
        const EdgeFlags = sys::EcsTableEdgeFlags;
        const AddEdgeFlags = sys::EcsTableAddEdgeFlags;
        const RemoveEdgeFlags = sys::EcsTableRemoveEdgeFlags;
    }
}

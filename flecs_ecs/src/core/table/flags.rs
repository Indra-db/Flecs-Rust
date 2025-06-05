use crate::sys;

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct TableFlags: u32 {
        const HasBuiltins = sys::EcsTableHasBuiltins as u32;
        const IsPrefab = sys::EcsTableIsPrefab as u32;
        const HasIsA = sys::EcsTableHasIsA as u32;
        const HasChildOf = sys::EcsTableHasChildOf as u32;
        const HasName = sys::EcsTableHasName as u32;
        const HasPairs = sys::EcsTableHasPairs as u32;
        const HasModule = sys::EcsTableHasModule as u32;
        const IsDisabled = sys::EcsTableIsDisabled as u32;
        const NotQueryable = sys::EcsTableNotQueryable as u32;
        const HasCtors = sys::EcsTableHasCtors as u32;
        const HasDtors = sys::EcsTableHasDtors as u32;
        const HasCopy = sys::EcsTableHasCopy as u32;
        const HasMove = sys::EcsTableHasMove as u32;
        const HasToggle = sys::EcsTableHasToggle as u32;
        const HasOverrides = sys::EcsTableHasOverrides as u32;
        const HasOnAdd = sys::EcsTableHasOnAdd as u32;
        const HasOnRemove = sys::EcsTableHasOnRemove as u32;
        const HasOnSet = sys::EcsTableHasOnSet as u32;
        const HasOnTableFill = sys::EcsTableHasOnTableFill as u32;
        const HasOnTableEmpty = sys::EcsTableHasOnTableEmpty as u32;
        const HasOnTableCreate = sys::EcsTableHasOnTableCreate as u32;
        const HasOnTableDelete = sys::EcsTableHasOnTableDelete as u32;
        const HasSparse = sys::EcsTableHasSparse as u32;
        const HasDontFragment = sys::EcsTableHasDontFragment as u32;
        const OverrideDontFragment = sys::EcsTableOverrideDontFragment as u32;
        const HasUnion = sys::EcsTableHasUnion as u32;
        const HasTraversable = sys::EcsTableHasTraversable as u32;
        const HasOrderedChildren = sys::EcsTableHasOrderedChildren as u32;
        const EdgeReparent = sys::EcsTableEdgeReparent as u32;
        const MarkedForDelete = sys::EcsTableMarkedForDelete as u32;
        const HasLifecycle = sys::EcsTableHasLifecycle as u32;
        const IsComplex = sys::EcsTableIsComplex as u32;
        const HasAddActions = sys::EcsTableHasAddActions as u32;
        const HasRemoveActions = sys::EcsTableHasRemoveActions as u32;
        const EdgeFlags = sys::EcsTableEdgeFlags as u32;
        const AddEdgeFlags = sys::EcsTableAddEdgeFlags as u32;
        const RemoveEdgeFlags = sys::EcsTableRemoveEdgeFlags as u32;
    }
}

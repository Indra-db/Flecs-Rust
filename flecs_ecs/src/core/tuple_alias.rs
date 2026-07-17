use core::any::TypeId;

use crate::core::{ComponentOrPairId, flecs};

/// Type-level identity of a tuple term. Two terms with equal `first`/`second`/`is_pair`
/// always resolve to the same runtime component id in any world, and distinct keys
/// (without wildcards) never do. All fields derive from monomorphized types, so
/// comparisons fold to constants in optimized builds.
#[derive(Clone, Copy)]
pub struct TermAliasKey {
    first: TypeId,
    second: TypeId,
    is_pair: bool,
    is_mut: bool,
}

impl TermAliasKey {
    #[inline(always)]
    pub fn new<T: ComponentOrPairId>(is_mut: bool) -> Self {
        Self {
            first: TypeId::of::<T::First>(),
            second: TypeId::of::<T::Second>(),
            is_pair: T::IS_PAIR,
            is_mut,
        }
    }

    #[inline(always)]
    fn same_id(&self, other: &Self) -> bool {
        self.is_pair == other.is_pair
            && self.first == other.first
            && self.second == other.second
    }

    #[inline(always)]
    fn is_dynamic(id: TypeId) -> bool {
        id == TypeId::of::<flecs::Wildcard>() || id == TypeId::of::<flecs::Any>()
    }

    #[inline(always)]
    fn could_alias(&self, other: &Self) -> bool {
        let self_any = !self.is_pair && Self::is_dynamic(self.first);
        let other_any = !other.is_pair && Self::is_dynamic(other.first);
        if self_any || other_any {
            return true;
        }
        if self.is_pair != other.is_pair {
            return false;
        }
        let first_overlap = self.first == other.first
            || Self::is_dynamic(self.first)
            || Self::is_dynamic(other.first);
        if !self.is_pair {
            return first_overlap;
        }
        let second_overlap = self.second == other.second
            || Self::is_dynamic(self.second)
            || Self::is_dynamic(other.second);
        first_overlap && second_overlap
    }
}

/// Returns the index of the first term whose id is guaranteed to equal an earlier
/// term's id while at least one of the two is mutable. Fully determined by types,
/// folds to a constant after monomorphization.
#[inline(always)]
pub fn static_alias_conflict(keys: &[TermAliasKey]) -> Option<usize> {
    let mut i = 1;
    while i < keys.len() {
        let mut j = 0;
        while j < i {
            if keys[j].same_id(&keys[i]) && (keys[i].is_mut || keys[j].is_mut) {
                return Some(i);
            }
            j += 1;
        }
        i += 1;
    }
    None
}

/// Whether any two terms could resolve to the same runtime id with mutable access
/// involved (duplicate keys, or wildcard/any terms). When false, per-iteration
/// pointer aliasing checks are provably unnecessary and the optimizer removes them.
#[inline(always)]
pub fn needs_runtime_alias_check(keys: &[TermAliasKey]) -> bool {
    let mut i = 1;
    while i < keys.len() {
        let mut j = 0;
        while j < i {
            if (keys[i].is_mut || keys[j].is_mut) && keys[j].could_alias(&keys[i]) {
                return true;
            }
            j += 1;
        }
        i += 1;
    }
    false
}

use core::any::TypeId;
use core::hash::BuildHasher;
use core::hash::Hasher;

pub(crate) type FlecsIdMap = hashbrown::HashMap<TypeId, u64, NoOpHash>;

// A hasher for `TypeId`s that takes advantage of its known characteristics.
// TypeIds are already a hash, so we can just use that.
#[derive(Debug, Default)]
#[doc(hidden)]
pub struct NoOpHasher(u64);

impl Hasher for NoOpHasher {
    fn write(&mut self, _: &[u8]) {
        unimplemented!("This NoOpHasher can only handle u64s")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

// A hasher builder that always returns the same hasher meant for `TypeId`s.
#[derive(Clone, Default)]
pub struct NoOpHash;

impl BuildHasher for NoOpHash {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher(0)
    }
}

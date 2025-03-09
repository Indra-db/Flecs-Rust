use dashmap::DashMap;
use flecs_ecs::sys;
use foldhash::fast::RandomState;
use smallvec::{SmallVec, smallvec};

use core::sync::atomic::{AtomicU16, Ordering};

use super::ReadWriteId;

/// Reserve the highest bit as the write flag.
const WRITE_FLAG: u16 = 1 << 15;
/// The remaining bits hold the read count.
const READ_MASK: u16 = WRITE_FLAG - 1;

type ComponentOrPairId = u64;
type TableId = u64;

// we use u128 over (u64, u64) to avoid the extra hash calculation and slightly better memory footprint.
type ComponentOrPairIdAndTableId = u128;

pub(crate) fn combone_ids(id: ComponentOrPairId, table_id: TableId) -> ComponentOrPairIdAndTableId {
    ((id as u128) << 64) | (table_id as u128)
}

pub(crate) struct ReadWriteCounter {
    counter: AtomicU16,
}

impl ReadWriteCounter {
    pub(crate) fn new() -> Self {
        Self {
            counter: AtomicU16::new(0),
        }
    }

    pub(crate) fn increment_read(&self) {
        loop {
            let curr = self.counter.load(Ordering::Relaxed);
            if curr & WRITE_FLAG != 0 {
                panic!("Cannot increment read: write already set");
            }

            if self
                .counter
                .compare_exchange_weak(curr, curr + 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub(crate) fn decrement_read(&self) {
        loop {
            let curr = self.counter.load(Ordering::Relaxed);

            debug_assert!(curr & READ_MASK != 0);

            if self
                .counter
                .compare_exchange_weak(curr, curr - 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub(crate) fn set_write(&self) {
        loop {
            let curr = self.counter.load(Ordering::Relaxed);
            if curr != 0 {
                panic!("Cannot set write: reads already present or write already set");
            }
            if self
                .counter
                .compare_exchange_weak(curr, WRITE_FLAG, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub(crate) fn clear_write(&self) {
        loop {
            let curr = self.counter.load(Ordering::Relaxed);
            debug_assert!(curr & WRITE_FLAG != 0);

            if self
                .counter
                .compare_exchange_weak(curr, 0, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }
}

//for bulk entity test where I'm catching the panic.
impl core::panic::RefUnwindSafe for ReadWriteComponentsMap {}

/// A thread-safe map to track entity access
pub(crate) struct ReadWriteComponentsMap {
    // Maps entity ID to number of readers
    pub(crate) read_write: DashMap<ComponentOrPairIdAndTableId, ReadWriteCounter, RandomState>,
}

impl ReadWriteComponentsMap {
    pub(crate) fn new() -> Self {
        Self {
            read_write: DashMap::with_hasher(RandomState::default()),
        }
    }

    pub(crate) fn add_entry_with(
        &self,
        id: ComponentOrPairIdAndTableId,
        counter: ReadWriteCounter,
    ) {
        self.read_write.insert(id, counter);
    }

    pub(crate) fn remove_entry(&self, id: ComponentOrPairId, table_id: TableId) {
        self.read_write.remove(&combone_ids(id, table_id));
    }

    pub(crate) fn increment_read(&self, id: ComponentOrPairId, table_id: TableId) {
        let id = combone_ids(id, table_id);
        if let Some(counter) = self.read_write.get(&id) {
            counter.increment_read();
        } else {
            let counter = ReadWriteCounter::new();
            counter.increment_read();
            self.add_entry_with(id, counter);
        }
    }

    pub(crate) fn decrement_read(&self, id: ComponentOrPairId, table_id: TableId) {
        if let Some(counter) = self.read_write.get(&combone_ids(id, table_id)) {
            counter.decrement_read();
        }
    }

    pub(crate) fn set_write(&self, id: ComponentOrPairId, table_id: TableId) {
        let id = combone_ids(id, table_id);
        if let Some(counter) = self.read_write.get(&id) {
            counter.set_write();
        } else {
            let counter = ReadWriteCounter::new();
            counter.set_write();
            self.add_entry_with(id, counter);
        }
    }

    pub(crate) fn clear_write(&self, id: ComponentOrPairId, table_id: TableId) {
        if let Some(counter) = self.read_write.get(&combone_ids(id, table_id)) {
            counter.clear_write();
        }
    }

    pub(crate) fn increment_counters_from_iter(
        &self,
        iter: &sys::ecs_iter_t,
    ) -> SmallVec<[u8; 10]> {
        let terms = unsafe { (*iter.query).terms };
        let terms_count = unsafe { (*iter.query).term_count };
        let ids = unsafe { core::slice::from_raw_parts(iter.ids, terms_count as usize) };
        let table_id = unsafe { sys::ecs_rust_table_id(iter.table) };
        // we don't expect more than 20 indices
        //TODO we can put this outside the while loop, optimize later
        let mut indices = smallvec![0_u8; 20_usize];
        for i in 0..terms_count as usize {
            let id = ids[i];
            if id == 0 {
                indices.push(i as u8);
                continue;
            }
            let term = terms[i];

            match term.inout as u32 {
                sys::ecs_inout_kind_t_EcsIn => {
                    self.increment_read(term.id, table_id);
                }
                sys::ecs_inout_kind_t_EcsInOut | sys::ecs_inout_kind_t_EcsOut => {
                    self.set_write(term.id, table_id);
                }
                _ => {}
            }
        }
        indices
    }

    pub(crate) fn decrement_counters_from_iter(&self, iter: &sys::ecs_iter_t) {
        let table_id = unsafe { sys::ecs_rust_table_id(iter.table) };
        let terms = unsafe { (*iter.query).terms };
        let terms_count = unsafe { (*iter.query).term_count };

        for term in terms.iter().take(terms_count as usize) {
            match term.inout as u32 {
                sys::ecs_inout_kind_t_EcsIn => {
                    self.decrement_read(term.id, table_id);
                }
                sys::ecs_inout_kind_t_EcsInOut | sys::ecs_inout_kind_t_EcsOut => {
                    self.clear_write(term.id, table_id);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn increment_counters_from_id(&self, id: ReadWriteId, table_id: TableId) {
        match id {
            ReadWriteId::Read(id) => {
                self.increment_read(id, table_id);
            }
            ReadWriteId::Write(id) => {
                self.set_write(id, table_id);
            }
        }
    }

    pub(crate) fn increment_counters_from_ids(&self, ids: &[ReadWriteId], table_id: TableId) {
        for id in ids {
            match id {
                ReadWriteId::Read(id) => {
                    self.increment_read(*id, table_id);
                }
                ReadWriteId::Write(id) => {
                    self.set_write(*id, table_id);
                }
            }
        }
    }

    pub(crate) fn decrement_counters_from_id(&self, id: ReadWriteId, table_id: TableId) {
        match id {
            ReadWriteId::Read(id) => {
                self.decrement_read(id, table_id);
            }
            ReadWriteId::Write(id) => {
                self.clear_write(id, table_id);
            }
        }
    }

    pub(crate) fn decrement_counters_from_ids(&self, ids: &[ReadWriteId], table_id: TableId) {
        for id in ids {
            match id {
                ReadWriteId::Read(id) => {
                    self.decrement_read(*id, table_id);
                }
                ReadWriteId::Write(id) => {
                    self.clear_write(*id, table_id);
                }
            }
        }
    }

    pub(crate) fn panic_if_any_write_is_set(&self, ids: &[u64], table_id: TableId) {
        for id in ids {
            if let Some(counter) = self.read_write.get(&combone_ids(*id, table_id)) {
                if counter.counter.load(Ordering::Relaxed) & WRITE_FLAG != 0 {
                    panic!("Write already set");
                }
            }
        }
    }
}

#[test]
fn read_write_counter() {
    dbg!(crate::core::InOutKind::InOut as u32);
    dbg!(crate::core::InOutKind::In as u32);
    dbg!(crate::core::InOutKind::Out as u32);
    let counter = ReadWriteCounter::new();
    counter.increment_read();
    counter.increment_read();
    counter.increment_read();
    counter.decrement_read();
    counter.decrement_read();
    counter.decrement_read();
    counter.set_write();
    counter.clear_write();
    counter.increment_read();
    counter.increment_read();
    counter.decrement_read();
    counter.decrement_read();
}

#[test]
#[should_panic]
fn read_write_counter_panic() {
    let counter = ReadWriteCounter::new();
    counter.increment_read();
    counter.set_write();
}

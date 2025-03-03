use dashmap::DashMap;
use flecs_ecs::sys;
use smallvec::{smallvec, SmallVec};

use core::sync::atomic::{AtomicU32, Ordering};

use super::ReadWriteId;

/// Reserve the highest bit as the write flag.
const WRITE_FLAG: u32 = 1 << 31;
/// The remaining bits hold the read count.
const READ_MASK: u32 = WRITE_FLAG - 1;

type EntityId = u64;
pub(crate) struct ReadWriteCounter {
    counter: AtomicU32,
}

impl ReadWriteCounter {
    pub(crate) fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
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
    pub(crate) read_write: DashMap<EntityId, ReadWriteCounter, super::NoOpHash>,
}

impl ReadWriteComponentsMap {
    pub(crate) fn new() -> Self {
        Self {
            read_write: Default::default(),
        }
    }

    pub(crate) fn add_entry(&self, entity: EntityId) {
        self.read_write.insert(entity, ReadWriteCounter::new());
    }

    pub(crate) fn add_entry_with(&self, entity: EntityId, counter: ReadWriteCounter) {
        self.read_write.insert(entity, counter);
    }

    pub(crate) fn remove_entry(&self, entity: EntityId) {
        self.read_write.remove(&entity);
    }

    pub(crate) fn increment_read(&self, entity: EntityId) {
        if let Some(counter) = self.read_write.get(&entity) {
            counter.increment_read();
        } else {
            let counter = ReadWriteCounter::new();
            counter.increment_read();
            self.add_entry_with(entity, counter);
        }
    }

    pub(crate) fn decrement_read(&self, entity: EntityId) {
        if let Some(counter) = self.read_write.get(&entity) {
            counter.decrement_read();
        }
    }

    pub(crate) fn set_write(&self, entity: EntityId) {
        if let Some(counter) = self.read_write.get(&entity) {
            counter.set_write();
        } else {
            let counter = ReadWriteCounter::new();
            counter.set_write();
            self.add_entry_with(entity, counter);
        }
    }

    pub(crate) fn clear_write(&self, entity: EntityId) {
        if let Some(counter) = self.read_write.get(&entity) {
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
        // we don't expect more than 20 indices
        let mut indices = smallvec![0_u8; 20 as usize];
        for i in 0..terms_count as usize {
            let id = ids[i];
            if id == 0 {
                indices.push(i as u8);
                continue;
            }
            let term = terms[i];

            match term.inout as u32 {
                sys::ecs_inout_kind_t_EcsIn => {
                    self.increment_read(term.id);
                }
                sys::ecs_inout_kind_t_EcsInOut | sys::ecs_inout_kind_t_EcsOut => {
                    self.set_write(term.id);
                }
                _ => {}
            }
        }
        indices
    }

    pub(crate) fn decrement_counters_from_iter(&self, iter: &sys::ecs_iter_t) {
        let terms = unsafe { (*iter.query).terms };
        let terms_count = unsafe { (*iter.query).term_count };

        for i in 0..terms_count as usize {
            let term = terms[i];

            match term.inout as u32 {
                sys::ecs_inout_kind_t_EcsIn => {
                    self.decrement_read(term.id);
                }
                sys::ecs_inout_kind_t_EcsInOut | sys::ecs_inout_kind_t_EcsOut => {
                    self.clear_write(term.id);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn increment_counters_from_id(&self, id: ReadWriteId) {
        match id {
            ReadWriteId::Read(id) => {
                self.increment_read(id);
            }
            ReadWriteId::Write(id) => {
                self.set_write(id);
            }
        }
    }

    pub(crate) fn increment_counters_from_ids(&self, ids: &[ReadWriteId]) {
        for id in ids {
            match id {
                ReadWriteId::Read(id) => {
                    self.increment_read(*id);
                }
                ReadWriteId::Write(id) => {
                    self.set_write(*id);
                }
            }
        }
    }

    pub(crate) fn decrement_counters_from_id(&self, id: ReadWriteId) {
        match id {
            ReadWriteId::Read(id) => {
                self.decrement_read(id);
            }
            ReadWriteId::Write(id) => {
                self.clear_write(id);
            }
        }
    }

    pub(crate) fn decrement_counters_from_ids(&self, ids: &[ReadWriteId]) {
        for id in ids {
            match id {
                ReadWriteId::Read(id) => {
                    self.decrement_read(*id);
                }
                ReadWriteId::Write(id) => {
                    self.clear_write(*id);
                }
            }
        }
    }

    pub(crate) fn panic_if_any_write_is_set(&self, ids: &[u64]) {
        for id in ids {
            if let Some(counter) = self.read_write.get(id) {
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

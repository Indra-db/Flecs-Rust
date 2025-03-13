use crate::core::{IdOperations, IdView};

use super::ReadWriteId;
use super::WorldRef;
use core::sync::atomic::{AtomicU16, Ordering};
use dashmap::DashMap;
use flecs_ecs::sys;
use flecs_ecs_sys::{ecs_table_column_lock_write_begin, ecs_table_column_lock_write_end};
use foldhash::fast::RandomState;
use smallvec::{SmallVec, smallvec};

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

    pub(crate) fn increment_read(&self) -> Result<(), ()> {
        loop {
            let curr = self.counter.load(Ordering::Relaxed);
            if curr & WRITE_FLAG != 0 {
                return Err(());
            }

            if self
                .counter
                .compare_exchange_weak(curr, curr + 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(());
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

    pub(crate) fn set_write(&self) -> Result<(), ()> {
        loop {
            let curr = self.counter.load(Ordering::Relaxed);
            if curr != 0 {
                return Err(());
            }
            if self
                .counter
                .compare_exchange_weak(curr, WRITE_FLAG, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(());
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

    pub(crate) fn increment_read(
        &self,
        comp_id: ComponentOrPairId,
        table_id: TableId,
        world: &WorldRef,
    ) {
        let id = combone_ids(comp_id, table_id);
        if let Some(counter) = self.read_write.get(&id) {
            if counter.increment_read().is_err() {
                panic!(
                    "Cannot increment read: write already set for component: {} with table id: {}",
                    {
                        let id = IdView::new_from_id(world, comp_id);
                        if id.is_pair() {
                            format!(
                                "({}, {})",
                                world.entity_from_id(id.first_id()),
                                world.entity_from_id(id.second_id())
                            )
                        } else {
                            format!("{}", id.entity_view())
                        }
                    },
                    table_id
                );
            }
        } else {
            let counter = ReadWriteCounter::new();
            let _ = counter.increment_read();
            self.add_entry_with(id, counter);
        }
    }

    pub(crate) fn decrement_read(&self, id: ComponentOrPairId, table_id: TableId) {
        if let Some(counter) = self.read_write.get(&combone_ids(id, table_id)) {
            counter.decrement_read();
        }
    }

    pub(crate) fn set_write(
        &self,
        comp_id: ComponentOrPairId,
        table_id: TableId,
        world: &WorldRef,
    ) {
        let id = combone_ids(comp_id, table_id);
        if let Some(counter) = self.read_write.get(&id) {
            if counter.set_write().is_err() {
                panic!(
                    "Cannot set write: reads already present or write already set for component: {} with table id: {}",
                    {
                        let id = IdView::new_from_id(world, comp_id);
                        if id.is_pair() {
                            format!(
                                "({}, {})",
                                world.entity_from_id(id.first_id()),
                                world.entity_from_id(id.second_id())
                            )
                        } else {
                            format!("{}", id.entity_view())
                        }
                    },
                    table_id
                );
            }
        } else {
            let counter = ReadWriteCounter::new();
            let _ = counter.set_write();
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
        world: &WorldRef,
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
                    self.increment_read(term.id, table_id, world);
                }
                sys::ecs_inout_kind_t_EcsInOut | sys::ecs_inout_kind_t_EcsOut => {
                    self.set_write(term.id, table_id, world);
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

    pub(crate) fn increment_counters_from_id(
        &self,
        id: ReadWriteId,
        table_id: TableId,
        world: &WorldRef,
    ) {
        match id {
            ReadWriteId::Read(id) => {
                self.increment_read(id, table_id, world);
            }
            ReadWriteId::Write(id) => {
                self.set_write(id, table_id, world);
            }
        }
    }

    pub(crate) fn increment_counters_from_ids(
        &self,
        ids: &[ReadWriteId],
        table_id: TableId,
        world: &WorldRef,
    ) {
        for id in ids {
            match id {
                ReadWriteId::Read(id) => {
                    self.increment_read(*id, table_id, world);
                }
                ReadWriteId::Write(id) => {
                    self.set_write(*id, table_id, world);
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

#[cfg(feature = "flecs_safety_readwrite_locks")]
pub(super) const INCREMENT: bool = true;
#[cfg(feature = "flecs_safety_readwrite_locks")]
pub(super) const DECREMENT: bool = false;

#[cfg(feature = "flecs_safety_readwrite_locks")]
pub(crate) fn do_read_write_locks<const INCREMENT: bool>(
    iter: &sys::ecs_iter_t,
    count: usize,
    world: &WorldRef,
) {
    unsafe {
        for i in 0..count {
            if !sys::ecs_field_is_set(iter, i as i8) {
                continue;
            }

            let tr = *iter.trs.add(i);

            // when it's a `not` term, the table does not have the component
            if tr.is_null() {
                continue;
            }

            let component_id = *iter.ids.add(i);

            if sys::ecs_id_is_wildcard(component_id) {
                continue;
            }
            let idr = (*tr).hdr.cache as *mut sys::ecs_id_record_t;

            // don't bother with tags, but we still need to lock the sparse components
            if (*tr).column == -1 {
                //sparse components are not stored in tables so check for that
                if sys::ecs_rust_is_sparse_idr(idr) {
                    if sys::ecs_field_is_readonly(iter, i as i8) {
                        if INCREMENT {
                            sparse_id_record_lock_read_begin(world, idr);
                        } else {
                            sparse_id_record_lock_read_end(idr);
                        }
                    } else if INCREMENT {
                        sparse_id_record_lock_write_begin(world, idr);
                    } else {
                        sparse_id_record_lock_write_end(idr);
                    }
                }
                continue;
            }

            let table = (*tr).hdr.table;

            if sys::ecs_field_is_readonly(iter, i as i8) {
                if INCREMENT {
                    table_column_lock_read_begin(world, table, (*tr).column, world.stage_id());
                } else {
                    table_column_lock_read_end(table, (*tr).column, world.stage_id());
                }
            } else if INCREMENT {
                table_column_lock_write_begin(world, table, (*tr).column, world.stage_id());
            } else {
                table_column_lock_write_end(table, (*tr).column, world.stage_id());
            }
        }
    }
}

fn component_id_from_table_column(table: *mut sys::ecs_table_t, column: i16) -> u64 {
    unsafe {
        *(*sys::ecs_table_get_type(table))
            .array
            .add(sys::ecs_table_column_to_type_index(table, column as i32) as usize)
    }
}

pub(crate) fn sparse_id_record_lock_read_begin(world: &WorldRef, idr: *mut sys::ecs_id_record_t) {
    unsafe {
        if sys::ecs_sparse_id_record_lock_read_begin(idr) {
            panic!(
                "Cannot increment read: write already set for component: {}",
                {
                    let id = IdView::new_from_id(world, sys::ecs_id_from_id_record(idr));
                    if id.is_pair() {
                        format!(
                            "({}, {})",
                            world.entity_from_id(id.first_id()),
                            world.entity_from_id(id.second_id())
                        )
                    } else {
                        format!("{}", id.entity_view())
                    }
                },
            );
        }
    }
}

pub(crate) fn sparse_id_record_lock_read_end(idr: *mut sys::ecs_id_record_t) {
    unsafe {
        sys::ecs_sparse_id_record_lock_read_end(idr);
    }
}

pub(crate) fn sparse_id_record_lock_write_begin(world: &WorldRef, idr: *mut sys::ecs_id_record_t) {
    unsafe {
        if sys::ecs_sparse_id_record_lock_write_begin(idr) {
            panic!(
                "Cannot set write: reads already present or write already set for component: {}",
                {
                    let id = IdView::new_from_id(world, sys::ecs_id_from_id_record(idr));
                    if id.is_pair() {
                        format!(
                            "({}, {})",
                            world.entity_from_id(id.first_id()),
                            world.entity_from_id(id.second_id())
                        )
                    } else {
                        format!("{}", id.entity_view())
                    }
                },
            );
        }
    }
}

pub(crate) fn sparse_id_record_lock_write_end(idr: *mut sys::ecs_id_record_t) {
    unsafe {
        sys::ecs_sparse_id_record_lock_write_end(idr);
    }
}

pub(crate) fn table_column_lock_read_begin(
    world: &WorldRef,
    table: *mut sys::ecs_table_t,
    column: i16,
    stage_id: i32,
) {
    unsafe {
        if sys::ecs_table_column_lock_read_begin(table, column, stage_id) {
            panic!(
                "Cannot increment read: write already set for component: {}",
                {
                    let id =
                        IdView::new_from_id(world, component_id_from_table_column(table, column));
                    if id.is_pair() {
                        format!(
                            "({}, {})",
                            world.entity_from_id(id.first_id()),
                            world.entity_from_id(id.second_id())
                        )
                    } else {
                        format!("{}", id.entity_view())
                    }
                },
            );
        }
    }
}

pub(crate) fn table_column_lock_read_end(table: *mut sys::ecs_table_t, column: i16, stage_id: i32) {
    unsafe {
        sys::ecs_table_column_lock_read_end(table, column, stage_id);
    }
}

pub(crate) fn table_column_lock_write_begin(
    world: &WorldRef,
    table: *mut sys::ecs_table_t,
    column: i16,
    stage_id: i32,
) {
    unsafe {
        if ecs_table_column_lock_write_begin(table, column, stage_id) {
            panic!(
                "Cannot set write: reads already present or write already set for component: {}",
                {
                    let id =
                        IdView::new_from_id(world, component_id_from_table_column(table, column));
                    if id.is_pair() {
                        format!(
                            "({}, {})",
                            world.entity_from_id(id.first_id()),
                            world.entity_from_id(id.second_id())
                        )
                    } else {
                        format!("{}", id.entity_view())
                    }
                },
            );
        }
    }
}

pub(crate) fn table_column_lock_write_end(
    table: *mut sys::ecs_table_t,
    column: i16,
    stage_id: i32,
) {
    unsafe {
        ecs_table_column_lock_write_end(table, column, stage_id);
    }
}

#[test]
fn read_write_counter() {
    let counter = ReadWriteCounter::new();
    assert!(counter.increment_read().is_ok());
    assert!(counter.increment_read().is_ok());
    assert!(counter.increment_read().is_ok());
    counter.decrement_read();
    counter.decrement_read();
    counter.decrement_read();
    assert!(counter.set_write().is_ok());
    counter.clear_write();
    assert!(counter.increment_read().is_ok());
    assert!(counter.increment_read().is_ok());
    counter.decrement_read();
    counter.decrement_read();
}

#[test]
fn read_write_counter_panic() {
    let counter = ReadWriteCounter::new();
    let _ = counter.increment_read();
    assert!(counter.set_write().is_err());
}

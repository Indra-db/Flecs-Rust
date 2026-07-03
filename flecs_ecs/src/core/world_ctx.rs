use super::{FlecsArray, FlecsIdMap, World};
use crate::sys;

use core::cell::Cell;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::vec;

pub(crate) struct WorldCtx {
    query_ref_count: Cell<i32>,
    pub(crate) components: FlecsIdMap,
    pub(crate) components_array: FlecsArray,
    is_panicking: Cell<bool>,
}

impl WorldCtx {
    pub(crate) fn new() -> Self {
        Self {
            query_ref_count: Cell::new(0),
            components: Default::default(),
            components_array: vec![0; 500],
            is_panicking: Cell::new(false),
        }
    }

    pub(crate) fn inc_query_ref_count(&self) {
        unsafe {
            if sys::ecs_os_has_threading() {
                if let Some(ainc) = sys::ecs_os_api.ainc_ {
                    ainc(self.query_ref_count.as_ptr());
                }
            } else {
                self.query_ref_count.set(self.query_ref_count.get() + 1);
            }
        }
    }

    pub(crate) fn dec_query_ref_count(&self) {
        unsafe {
            if sys::ecs_os_has_threading() {
                if let Some(adec) = sys::ecs_os_api.adec_ {
                    adec(self.query_ref_count.as_ptr());
                }
            } else {
                self.query_ref_count.set(self.query_ref_count.get() - 1);
            }
        }
    }

    #[allow(dead_code)] //used in tests
    pub(crate) fn query_ref_count(&self) -> i32 {
        self.query_ref_count.get()
    }

    pub(crate) fn is_ref_count_zero(&self) -> bool {
        self.query_ref_count.get() == 0
    }

    pub(crate) fn set_is_panicking_true(&self) {
        self.is_panicking.set(true);
    }

    pub(crate) fn is_panicking(&self) -> bool {
        self.is_panicking.get() || std::thread::panicking()
    }
}

impl World {
    pub(crate) fn world_ctx(&self) -> &WorldCtx {
        unsafe { &*(sys::ecs_get_binding_ctx(self.raw_world.as_ptr()) as *const WorldCtx) }
    }
}

#[test]
fn query_ref_count() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }
    use flecs_ecs::core::*;
    use flecs_ecs::macros::*;

    #[derive(Component)]
    struct Tag;

    let world = World::new();
    let query = world.query::<()>().with(Tag).build();

    assert_eq!(world.world_ctx().query_ref_count(), 1);
    assert_eq!(query.reference_count(), 1);

    let query2 = query.clone();

    assert_eq!(world.world_ctx().query_ref_count(), 2);
    assert_eq!(query.reference_count(), 2);

    drop(query);

    assert_eq!(world.world_ctx().query_ref_count(), 1);
    assert_eq!(query2.reference_count(), 1);

    drop(query2);

    assert_eq!(world.world_ctx().query_ref_count(), 0);
}

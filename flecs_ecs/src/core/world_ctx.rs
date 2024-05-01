use super::World;
use crate::sys;

pub(crate) struct WorldCtx {
    query_ref_count: i32,
}

impl WorldCtx {
    pub(crate) fn new() -> Self {
        Self { query_ref_count: 0 }
    }

    pub(crate) fn inc_query_ref_count(&mut self) {
        unsafe {
            if sys::ecs_os_has_threading() {
                if let Some(ainc) = sys::ecs_os_api.ainc_ {
                    ainc(&mut self.query_ref_count);
                }
            } else {
                self.query_ref_count += 1;
            }
        }
    }

    pub(crate) fn dec_query_ref_count(&mut self) {
        unsafe {
            if sys::ecs_os_has_threading() {
                if let Some(adec) = sys::ecs_os_api.adec_ {
                    adec(&mut self.query_ref_count);
                }
            } else {
                self.query_ref_count -= 1;
            }
        }
    }

    pub(crate) fn query_ref_count(&self) -> i32 {
        self.query_ref_count
    }

    pub(crate) fn is_ref_count_zero(&self) -> bool {
        self.query_ref_count == 0
    }
}

pub(crate) extern "C" fn world_ctx_destruct(ctx: *mut std::ffi::c_void) {
    let ctx = unsafe { Box::from_raw(ctx as *mut WorldCtx) };
    drop(ctx);
}

impl World {
    pub(crate) fn world_ctx(&self) -> &WorldCtx {
        unsafe { &*(sys::ecs_get_binding_ctx(self.raw_world.as_ptr()) as *const WorldCtx) }
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn world_ctx_mut(&self) -> &mut WorldCtx {
        unsafe { &mut *(sys::ecs_get_binding_ctx(self.raw_world.as_ptr()) as *mut WorldCtx) }
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
    let query = world.query::<&Tag>().build();

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

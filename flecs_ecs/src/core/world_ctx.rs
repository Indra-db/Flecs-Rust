use super::World;
use crate::sys;

pub(crate) struct WorldCtx {
    query_ref_count: u32,
}

impl WorldCtx {
    pub(crate) fn new() -> Self {
        Self { query_ref_count: 0 }
    }

    pub(crate) fn inc_query_ref_count(&mut self) {
        self.query_ref_count += 1;
    }

    pub(crate) fn dec_query_ref_count(&mut self) {
        self.query_ref_count -= 1;
    }

    pub(crate) fn query_ref_count(&self) -> u32 {
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

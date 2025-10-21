//! Rest module component

use super::*;
// REST module components
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rest {
    #[doc = "< Port of server (optional, default = 27750)"]
    pub port: u16,
    #[doc = "< Interface address (optional, default = 0.0.0.0)"]
    pub ipaddr: *mut ::core::ffi::c_char,
    pub impl_: *mut ::core::ffi::c_void,
}

impl Default for Rest {
    fn default() -> Self {
        Self {
            port: Default::default(),
            ipaddr: core::ptr::null_mut::<core::ffi::c_char>(),
            impl_: core::ptr::null_mut::<core::ffi::c_void>(),
        }
    }
}

impl_component_traits_binding_type_w_id!(Rest, ECS_REST);
unsafe impl Send for Rest {}
unsafe impl Sync for Rest {}

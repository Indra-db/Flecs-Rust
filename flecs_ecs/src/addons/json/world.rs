use super::*;
use crate::core::*;
use crate::sys;
use alloc::string::String;
use alloc::string::ToString;

impl World {
    /// Serialize untyped value to JSON.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn to_json_id(&self, tid: impl IntoId, value: *const core::ffi::c_void) -> String {
        let tid: u64 = *tid.into_id(self);
        let world = self.world_ptr();
        unsafe {
            let json_ptr = sys::ecs_ptr_to_json(world, tid, value);
            let json = core::ffi::CStr::from_ptr(json_ptr)
                .to_string_lossy()
                .into_owned();
            sys::ecs_os_api.free_.expect("os api is missing")(json_ptr as *mut core::ffi::c_void);
            json
        }
    }

    /// Serialize value to JSON.
    pub fn to_json<'a, T: ComponentOrPairId>(&'a self, value: &'a T::CastType) -> String {
        self.to_json_id(
            T::get_id(self),
            value as *const T::CastType as *const core::ffi::c_void,
        )
    }

    /// Serialize value to JSON.
    pub fn to_json_dyn<'a, T>(&'a self, id: FetchedId<T>, value: &'a T) -> String {
        self.to_json_id(id.id(), value as *const T as *const core::ffi::c_void)
    }

    /// Serialize world to JSON.
    pub fn to_json_world(&self, desc: Option<&WorldToJsonDesc>) -> String {
        let world = self.world_ptr_mut();
        let desc_ptr = desc
            .map(|d| d as *const WorldToJsonDesc)
            .unwrap_or(core::ptr::null());

        unsafe {
            let json_ptr = sys::ecs_world_to_json(world, desc_ptr);
            let json = core::ffi::CStr::from_ptr(json_ptr)
                .to_str()
                .unwrap()
                .to_string();
            sys::ecs_os_api.free_.expect("os api is missing")(json_ptr as *mut core::ffi::c_void);
            json
        }
    }

    /// Deserialize value from JSON.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn from_json_id(
        &self,
        tid: impl IntoId,
        value: *mut core::ffi::c_void,
        json: &str,
        desc: Option<&FromJsonDesc>,
    ) {
        let tid: u64 = *tid.into_id(self);
        let world = self.ptr_mut();
        let desc_ptr = desc
            .map(|d| d as *const FromJsonDesc)
            .unwrap_or(core::ptr::null());
        //TODO json object to prevent multiple conversions
        let json = compact_str::format_compact!("{}\0", json);

        unsafe {
            sys::ecs_ptr_from_json(world, tid, value, json.as_ptr() as *const _, desc_ptr);
        }
    }

    /// Deserialize value from JSON.
    pub fn from_json<T: ComponentOrPairId>(
        &self,
        value: &mut T::CastType,
        json: &str,
        desc: Option<&FromJsonDesc>,
    ) {
        self.from_json_id(
            T::CastType::get_id(self),
            value as *mut T::CastType as *mut core::ffi::c_void,
            json,
            desc,
        );
    }

    /// Deserialize JSON into world.
    pub fn from_json_world(&self, json: &str, desc: Option<&FromJsonDesc>) -> &Self {
        let world = self.ptr_mut();
        //TODO json object to prevent multiple conversions
        let json = compact_str::format_compact!("{}\0", json);
        let desc_ptr = desc
            .map(|d| d as *const FromJsonDesc)
            .unwrap_or(core::ptr::null());

        unsafe {
            sys::ecs_world_from_json(world, json.as_ptr() as *const _, desc_ptr);
        }

        self
    }

    /// Deserialize JSON file into world.
    pub fn from_json_world_file(
        &mut self,
        json_file: &str,
        desc: Option<&FromJsonDesc>,
    ) -> &mut Self {
        let world = self.ptr_mut();
        //TODO json object to prevent multiple conversions
        let json_file = compact_str::format_compact!("{}\0", json_file);
        let desc_ptr = desc
            .map(|d| d as *const FromJsonDesc)
            .unwrap_or(core::ptr::null());

        unsafe {
            sys::ecs_world_from_json_file(world, json_file.as_ptr() as *const _, desc_ptr);
        }

        self
    }
}

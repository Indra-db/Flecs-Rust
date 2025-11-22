use super::*;
use crate::core::*;
use crate::sys;
use alloc::string::String;
use alloc::string::ToString;

impl World {
    pub(crate) unsafe fn to_json_id_internal<T: IntoId, const DO_CHECKS: bool>(
        &self,
        tid: T,
        value: *const <T as IntoId>::CastType,
    ) -> Option<String> {
        if DO_CHECKS {
            assert!(!value.is_null(), "value pointer is null");
        }
        let tid: u64 = *tid.into_id(self);
        let world = self.world_ptr();
        unsafe {
            let json_ptr = sys::ecs_ptr_to_json(world, tid, value as *const core::ffi::c_void);
            if DO_CHECKS && json_ptr.is_null() {
                return None;
            }
            let json = core::ffi::CStr::from_ptr(json_ptr)
                .to_string_lossy()
                .into_owned();
            sys::ecs_os_api.free_.expect("os api is missing")(json_ptr as *mut core::ffi::c_void);
            Some(json)
        }
    }

    /// Serialize untyped value to JSON.
    ///
    /// # Arguments
    ///
    /// * `tid` - The type id of the value to serialize
    /// * `value` - Raw pointer to the value to serialize
    ///
    /// # Returns
    ///
    /// * `Some(String)` - JSON representation of the value if serialization succeeds
    /// * `None` - If the value pointer is null or serialization fails
    ///
    /// # Safety
    ///
    /// The caller must ensure that `value` points to valid data of the type specified by `tid`,
    /// or is null. This function will safely handle null pointers by returning `None`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = 42i32;
    /// let json = world.to_json_id(type_id, &value as *const _ as *const c_void);
    /// assert_eq!(json, Some("42".to_string()));
    ///
    /// // Null pointer returns None
    /// let json = world.to_json_id(type_id, core::ptr::null());
    /// assert_eq!(json, None);
    /// ```
    pub unsafe fn to_json_id<T: IntoId>(
        &self,
        tid: T,
        value: *const <T as IntoId>::CastType,
    ) -> Option<String> {
        unsafe { self.to_json_id_internal::<T, true>(tid, value) }
    }

    /// Serialize value to JSON.
    ///
    /// This is a type-safe wrapper around [`to_json_id`](Self::to_json_id) that accepts a reference
    /// instead of a raw pointer.
    ///
    /// # Returns
    ///
    /// A JSON string representation of the value. Since a valid reference is provided,
    /// this will always succeed and return a `String`.
    pub fn to_json<'a, T: ComponentOrPairId>(&'a self, value: &'a T::CastType) -> String {
        unsafe {
            self.to_json_id_internal::<u64, false>(
                T::get_id(self),
                value as *const T::CastType as *const core::ffi::c_void,
            )
            .expect("to_json should not fail with a valid reference")
        }
    }

    /// Serialize value to JSON.
    ///
    /// This is a type-safe wrapper around [`to_json_id`](Self::to_json_id) that accepts a reference
    /// instead of a raw pointer.
    ///
    /// # Returns
    ///
    /// A JSON string representation of the value. Since a valid reference is provided,
    /// this will always succeed and return a `String`.
    pub fn to_json_dyn<'a, T>(&'a self, id: FetchedId<T>, value: &'a T) -> String {
        unsafe {
            self.to_json_id(id.id(), value as *const T as *const core::ffi::c_void)
                .expect("to_json_dyn should not fail with a valid reference")
        }
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

    #[allow(clippy::wrong_self_convention)]
    pub(crate) unsafe fn from_json_id_internal<T: IntoId, const DO_CHECKS: bool>(
        &self,
        tid: T,
        value: *mut <T as IntoId>::CastType,
        json: &str,
        desc: Option<&FromJsonDesc>,
    ) -> bool {
        if DO_CHECKS {
            assert!(!value.is_null(), "value pointer is null");
        }
        let tid: u64 = *tid.into_id(self);
        let world = self.ptr_mut();
        let desc_ptr = desc
            .map(|d| d as *const FromJsonDesc)
            .unwrap_or(core::ptr::null());
        //TODO json object to prevent multiple conversions
        let json = compact_str::format_compact!("{}\0", json);

        unsafe {
            let result = sys::ecs_ptr_from_json(
                world,
                tid,
                value as *mut core::ffi::c_void,
                json.as_ptr() as *const _,
                desc_ptr,
            );
            if DO_CHECKS { !result.is_null() } else { true }
        }
    }

    /// Deserialize value from JSON.
    ///
    /// # Arguments
    ///
    /// * `tid` - The type id of the value to deserialize
    /// * `value` - Raw pointer to the memory to write to
    /// * `json` - The JSON expression to parse
    /// * `desc` - Optional configuration parameters for deserializer
    ///
    /// # Returns
    ///
    /// * `true` - If deserialization succeeds
    /// * `false` - If the value pointer is null or deserialization fails
    ///
    /// # Safety
    ///
    /// The caller must ensure that `value` points to valid, initialized memory large enough
    /// to contain a value of the type specified by `tid`, or is null. This function will
    /// safely handle null pointers by returning `false`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut value = 0i32;
    /// let success = world.from_json_id(type_id, &mut value as *mut _ as *mut c_void, "42", None);
    /// assert!(success);
    /// assert_eq!(value, 42);
    ///
    /// // Null pointer returns false
    /// let success = world.from_json_id(type_id, core::ptr::null_mut(), "42", None);
    /// assert!(!success);
    /// ```
    pub unsafe fn from_json_id<T: IntoId>(
        &self,
        tid: T,
        value: *mut <T as IntoId>::CastType,
        json: &str,
        desc: Option<&FromJsonDesc>,
    ) -> bool {
        unsafe { self.from_json_id_internal::<T, true>(tid, value, json, desc) }
    }

    /// Deserialize value from JSON.
    ///
    /// This is a type-safe wrapper around [`from_json_id`](Self::from_json_id) that accepts a mutable reference
    /// instead of a raw pointer.
    ///
    /// # Panics
    ///
    /// Panics if deserialization fails. Since a valid reference is provided, failures indicate
    /// invalid JSON or type mismatches.
    pub fn from_json<T: ComponentOrPairId>(
        &self,
        value: &mut T::CastType,
        json: &str,
        desc: Option<&FromJsonDesc>,
    ) {
        unsafe {
            self.from_json_id_internal::<u64, false>(
                T::CastType::get_id(self),
                value as *mut T::CastType as *mut core::ffi::c_void,
                json,
                desc,
            )
        };
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

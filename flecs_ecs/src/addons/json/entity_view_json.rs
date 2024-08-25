use super::{
    ComponentId, ComponentOrPairId, Entity, EntityId, EntityToJsonDesc, FromJsonDesc, IntoId,
    WorldProvider,
};
use crate::{
    core::{ecs_assert, FlecsErrorCode},
    sys,
};
pub trait EntityViewJson<'w>: EntityId + WorldProvider<'w> + Sized {
    /// Set component or pair id from JSON.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_json`
    #[doc(alias = "entity_builder::set_json")]
    fn set_json_id(self, comp: impl IntoId, json: &str, desc: Option<&FromJsonDesc>) -> Self {
        let comp: u64 = *comp.into();
        let world = self.world_ptr_mut();
        let id = *self.entity_id();
        unsafe {
            let type_ = sys::ecs_get_typeid(world, comp);
            if type_ == 0 {
                //sys::ecs_err(b"id is not a type\0".as_ptr() as *const i8);
                //TODO implement ecs_err
                return self;
            }

            let ptr = sys::ecs_ensure_id(world, id, comp);
            ecs_assert!(
                !ptr.is_null(),
                FlecsErrorCode::InternalError,
                "could not add comp to entity"
            );
            let json = compact_str::format_compact!("{}\0", json);
            if let Some(desc) = desc {
                sys::ecs_ptr_from_json(world, type_, ptr, json.as_ptr() as *const i8, desc);
            } else {
                sys::ecs_ptr_from_json(
                    world,
                    type_,
                    ptr,
                    json.as_ptr() as *const i8,
                    std::ptr::null(),
                );
            }
            sys::ecs_modified_id(world, id, comp);
        }
        self
    }

    /// Set component or pair from JSON.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_json`
    #[doc(alias = "entity_builder::set_json")]
    fn set_json<T: ComponentOrPairId>(self, json: &str, desc: Option<&FromJsonDesc>) -> Self {
        let world = self.world();
        self.set_json_id(T::get_id(world), json, desc)
    }

    /// Set pair from JSON where First is a type and Second is an entity id.
    fn set_json_first<Rel: ComponentId>(
        self,
        target: impl Into<Entity> + Copy,
        json: &str,
        desc: Option<&FromJsonDesc>,
    ) -> Self {
        let world = self.world();
        self.set_json_id((Rel::get_id(world), target), json, desc)
    }

    /// Set pair from JSON where First is an entity id and Second is a type.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::set_json_second`
    #[doc(alias = "entity_builder::set_json_second")]
    fn set_json_second<Target: ComponentId>(
        self,
        rel: impl Into<Entity> + Copy,
        json: &str,
        desc: Option<&FromJsonDesc>,
    ) -> Self {
        let world = self.world();
        self.set_json_id((rel, Target::get_id(world)), json, desc)
    }

    /// Serialize entity to JSON.
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::to_json`
    #[doc(alias = "entity_view::to_json")]
    fn to_json(&self, desc: Option<&EntityToJsonDesc>) -> String {
        let world = self.world_ptr();
        let id = *self.entity_id();
        let desc_ptr = desc
            .map(|d| d as *const EntityToJsonDesc)
            .unwrap_or(std::ptr::null());

        unsafe {
            let json_ptr = sys::ecs_entity_to_json(world, id, desc_ptr);
            let json = std::ffi::CStr::from_ptr(json_ptr)
                .to_str()
                .unwrap()
                .to_string();
            sys::ecs_os_api.free_.expect("os api is missing")(json_ptr as *mut std::ffi::c_void);
            json
        }
    }

    /// Deserialize entity to JSON.
    ///
    /// # See also
    ///
    /// * C++ API: `entity::from_json`
    #[doc(alias = "entity::from_json")]
    fn from_json(self, json: &str) -> Self {
        let world = self.world_ptr_mut();
        let id = *self.entity_id();
        //TODO we should have an Json Type so we don't need to make these conversions multiple times.
        let json = compact_str::format_compact!("{}\0", json);
        unsafe {
            sys::ecs_entity_from_json(world, id, json.as_ptr() as *const i8, std::ptr::null());
        }
        self
    }
}

use super::*;
use crate::core::*;
use core::ffi::CStr;

impl World {
    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    pub fn component_ext<T>(&self, id: FetchedId<T>) -> Component<'_, T> {
        Component::<T>::new_id(self, id)
    }

    /// Find or register component and set the name if not already set.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the component.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    pub fn component_named_ext<'a, T>(&'a self, id: FetchedId<T>, name: &str) -> Component<'a, T> {
        Component::<T>::new_named_id(self, id, name)
    }

    /// Return meta cursor to value
    pub fn cursor<T: ComponentId>(&self, data: &mut T) -> Cursor<'_> {
        let type_id = T::get_id(self.world());
        Cursor::new(self, type_id, data as *mut T as *mut c_void)
    }

    /// Return meta cursor to value
    pub fn cursor_id(&self, type_id: impl IntoEntity, ptr: *mut c_void) -> Cursor<'_> {
        if ptr.is_null() {
            panic!("ptr is null");
        }

        Cursor::new(self, type_id, ptr)
    }

    /// Create primitive type
    pub fn primitive(&self, kind: EcsPrimitiveKind) -> EntityView<'_> {
        let desc = sys::ecs_primitive_desc_t {
            kind: kind as sys::ecs_primitive_kind_t,
            entity: 0u64,
        };

        let eid = unsafe { sys::ecs_primitive_init(self.ptr_mut(), &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create primitive type"
        );
        EntityView::new_from(self, eid)
    }

    /// Create array type
    pub fn array(&self, elem_id: impl IntoEntity, array_count: i32) -> EntityView<'_> {
        let desc = sys::ecs_array_desc_t {
            type_: *elem_id.into_entity(self),
            count: array_count,
            entity: 0u64,
        };

        let eid = unsafe { sys::ecs_array_init(self.ptr_mut(), &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create array type"
        );
        EntityView::new_from(self, eid)
    }

    /// Create vector type
    pub fn vector_id(&self, elem_id: impl Into<Entity>) -> EntityView<'_> {
        let elem_id: u64 = *elem_id.into();
        let name_elem = unsafe { sys::ecs_get_name(self.world_ptr(), elem_id) };
        let cstr_name = unsafe { CStr::from_ptr(name_elem) };
        let name =
            compact_str::format_compact!("flecs::meta::vector::{}\0", cstr_name.to_string_lossy());
        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            parent: 0,
            symbol: core::ptr::null(),
            use_low_id: false,
            add: core::ptr::null(),
            add_expr: core::ptr::null(),
            set: core::ptr::null(),
        };
        let id = unsafe { sys::ecs_entity_init(self.world_ptr_mut(), &desc) };

        let desc = sys::ecs_vector_desc_t {
            entity: id,
            type_: elem_id,
        };

        let eid = unsafe { sys::ecs_vector_init(self.ptr_mut(), &desc) };

        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create vector type"
        );

        EntityView::new_from(self, eid)
    }

    /// Create vector type
    pub fn vector<T: 'static>(&self) -> EntityView<'_> {
        let id = self.component_id_map::<T>();
        self.vector_id(id)
    }
}

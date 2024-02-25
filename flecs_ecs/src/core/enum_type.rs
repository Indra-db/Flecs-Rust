use std::ffi::CStr;

use super::{
    c_types::{EntityT, WorldT},
    component_registration::{try_register_enum_component, ComponentType, Enum},
};

pub trait CachedEnumData: ComponentType<Enum> {
    const SIZE_ENUM_FIELDS: u32;
    type VariantIterator: Iterator<Item = Self>;

    /// ### Note
    /// this function is used to pass the name to the C API.
    fn get_cstr_name(&self) -> &CStr;

    fn get_enum_index(&self) -> usize;

    fn iter() -> Self::VariantIterator;

    /// # Note
    /// it only means that the enum is registered with a particular world, not necessarily yours.
    fn are_fields_registered_as_entities() -> bool {
        let mut result = true;
        let ptr = Self::__get_enum_data_ptr_mut();
        for i in 0..Self::SIZE_ENUM_FIELDS {
            unsafe {
                if *ptr.add(i as usize) == 0 {
                    result = false;
                    break;
                }
            }
        }
        result
    }

    fn is_field_registered_as_entity(&self) -> bool {
        let index = self.get_enum_index();
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) != 0 }
    }

    fn is_index_registered_as_entity(index: usize) -> bool {
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) != 0 }
    }

    fn get_entity_id_from_enum_field(&self, world: *mut WorldT) -> EntityT {
        try_register_enum_component::<Self>(world);
        let index = self.get_enum_index();
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) }
    }

    /// ## Safety
    /// this function assumes you're sure that the enum fields are registered previously
    /// if uncertain use get_entity_id_from_enum_field
    unsafe fn get_entity_id_from_enum_field_unchecked(&self) -> EntityT {
        let index = self.get_enum_index();
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) }
    }

    /// ## Safety
    /// This function is unsafe because it dereferences a raw pointer and you must ensure that the
    /// index is within the bounds of the number of variants in the enum.
    /// if uncertain, use SIZE_ENUM_FIELDS to check the number of variants.
    unsafe fn get_entity_id_from_enum_field_index(index: usize) -> u64 {
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) }
    }

    #[doc(hidden)]
    fn __get_enum_data_ptr_mut() -> *mut u64;
}

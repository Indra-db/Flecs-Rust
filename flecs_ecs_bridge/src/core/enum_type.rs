use std::ffi::CStr;

use super::{
    c_types::{EntityT, WorldT},
    component::{try_register_enum_component, CachedComponentData, ComponentType, Enum},
};

pub trait CachedEnumData: ComponentType<Enum> {
    const SIZE_ENUM_FIELDS: u32;
    type VariantIterator: Iterator<Item = Self>;

    /// # Note
    /// this function is used to pass the name to the C API.
    fn get_cstr_name(&self) -> &CStr;

    fn get_enum_index(&self) -> usize;

    fn iter() -> Self::VariantIterator;

    fn are_fields_registered_as_entities() -> bool {
        // when the enum is registered, the fields are registered as entities
        // and any entity id stored in the array should not be 0
        // as 0 represents an invalid entity id.
        // not the most elegant solution, but it works. (temporarily)
        unsafe { *Self::__get_enum_data_ptr_mut() != 0 }
    }

    fn get_entity_id_from_enum_field(&self, world: *mut WorldT) -> EntityT {
        //try_register_enum_component::<Self>(world); //TODO evaluate if we actually need this
        let index = self.get_enum_index();
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) }
    }

    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer and you must ensure that the
    /// index is within the bounds of the number of variants in the enum.
    /// if uncertain, use SIZE_ENUM_FIELDS to check the number of variants.
    unsafe fn get_entity_id_from_enum_field_index(index: usize) -> u64 {
        unsafe { *Self::__get_enum_data_ptr_mut().add(index) }
    }

    #[doc(hidden)]
    fn __get_enum_data_ptr_mut() -> *mut u64;
}

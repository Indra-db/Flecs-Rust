use std::ffi::CStr;

use super::component::CachedComponentData;

pub trait CachedEnumData: Clone + Default + CachedComponentData {
    //const SIZE_ENUM_FIELDS: u32;

    fn get_cstr_name(&self) -> &CStr;

    fn get_enum_index(&self) -> usize;
}

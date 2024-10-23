use core::ffi::CStr;
use flecs_ecs::prelude::*;
use flecs_ecs::sys;

macro_rules! generate_vec_meta_registration {
    ($world:ident, $($t:ty),*) => {
        $(
            {
                let id = id!($world, Vec<$t>);
                $world
                    .component_named_ext::<Vec<$t>>(id, format!("vector::{}", stringify!($t)).as_str())
                    .opaque_func_id::<_, $t>(id, meta_register_vector_default::<$t>);
            }
        )*
    };
}

pub(crate) fn meta_init_builtin(world: &World) {
    world
        .component_named::<String>("flecs::rust::String")
        .opaque_func(std_string_support);

    use std::any::TypeId;
    let map = world.components_map();

    map.insert(TypeId::of::<bool>(), ECS_BOOL_T);
    map.insert(TypeId::of::<char>(), ECS_CHAR_T);
    map.insert(TypeId::of::<u8>(), ECS_U8_T);
    map.insert(TypeId::of::<u16>(), ECS_U16_T);
    map.insert(TypeId::of::<u32>(), ECS_U32_T);
    map.insert(TypeId::of::<u64>(), ECS_U64_T);
    map.insert(TypeId::of::<usize>(), ECS_UPTR_T);
    map.insert(TypeId::of::<i8>(), ECS_I8_T);
    map.insert(TypeId::of::<i16>(), ECS_I16_T);
    map.insert(TypeId::of::<i32>(), ECS_I32_T);
    map.insert(TypeId::of::<i64>(), ECS_I64_T);
    map.insert(TypeId::of::<isize>(), ECS_IPTR_T);
    map.insert(TypeId::of::<f32>(), ECS_F32_T);
    map.insert(TypeId::of::<f64>(), ECS_F64_T);
    map.insert(TypeId::of::<Entity>(), ECS_ENTITY_T);

    map.insert(TypeId::of::<flecs::meta::Bool>(), ECS_BOOL_T);
    map.insert(TypeId::of::<flecs::meta::Char>(), ECS_CHAR_T);
    map.insert(TypeId::of::<flecs::meta::Byte>(), ECS_BYTE_T);
    map.insert(TypeId::of::<flecs::meta::U8>(), ECS_U8_T);
    map.insert(TypeId::of::<flecs::meta::U16>(), ECS_U16_T);
    map.insert(TypeId::of::<flecs::meta::U32>(), ECS_U32_T);
    map.insert(TypeId::of::<flecs::meta::U64>(), ECS_U64_T);
    map.insert(TypeId::of::<flecs::meta::UPtr>(), ECS_UPTR_T);
    map.insert(TypeId::of::<flecs::meta::I8>(), ECS_I8_T);
    map.insert(TypeId::of::<flecs::meta::I16>(), ECS_I16_T);
    map.insert(TypeId::of::<flecs::meta::I32>(), ECS_I32_T);
    map.insert(TypeId::of::<flecs::meta::I64>(), ECS_I64_T);
    map.insert(TypeId::of::<flecs::meta::IPtr>(), ECS_IPTR_T);
    map.insert(TypeId::of::<flecs::meta::F32>(), ECS_F32_T);
    map.insert(TypeId::of::<flecs::meta::F64>(), ECS_F64_T);
    map.insert(TypeId::of::<flecs::meta::String>(), ECS_STRING_T);
    map.insert(TypeId::of::<flecs::meta::Entity>(), ECS_ENTITY_T);
    map.insert(TypeId::of::<flecs::meta::Constant>(), ECS_CONSTANT);
    map.insert(TypeId::of::<flecs::meta::Quantity>(), ECS_QUANTITY);
    map.insert(TypeId::of::<flecs::meta::EcsOpaque>(), ECS_OPAQUE);

    generate_vec_meta_registration!(
        world, String, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, char, usize, isize
    );
}

fn std_string_support(world: WorldRef) -> Opaque<String> {
    let mut ts = Opaque::<String>::new(world);

    // Let reflection framework know what kind of type this is
    ts.as_type(flecs::meta::String);

    // Forward std::string value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &String| {
        let data = compact_str::format_compact!("{}\0", data);
        s.value_id(
            flecs::meta::String,
            &data.as_ptr() as *const *const u8 as *const std::ffi::c_void,
        )
    });

    // Serialize string into std::string
    ts.assign_string(|data: &mut String, value: *const std::ffi::c_char| {
        *data = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() }
    });

    ts
}

pub fn meta_ser_stringify_type_debug<T: core::fmt::Debug>(world: WorldRef) -> Opaque<T> {
    let mut ts = Opaque::<T>::new(world);

    // Let reflection framework know what kind of type this is
    ts.as_type(flecs::meta::String);

    // Forward std::string value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &T| {
        let data = format!("{:?}", data);
        let data = compact_str::format_compact!("{}\0", data);
        s.value_id(
            flecs::meta::String,
            &data.as_ptr() as *const *const u8 as *const std::ffi::c_void,
        )
    });

    ts
}

pub fn meta_ser_stringify_type_display<T: core::fmt::Display>(world: WorldRef) -> Opaque<T> {
    let mut ts = Opaque::<T>::new(world);

    // Let reflection framework know what kind of type this is
    ts.as_type(flecs::meta::String);

    // Forward std::string value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &T| {
        let data = format!("{}", data);
        let data = compact_str::format_compact!("{}\0", data);
        s.value_id(
            flecs::meta::String,
            &data.as_ptr() as *const *const u8 as *const std::ffi::c_void,
        )
    });

    ts
}

pub fn meta_register_vector_default<T: Default>(world: WorldRef) -> Opaque<Vec<T>, T> {
    let mut ts = Opaque::<Vec<T>, T>::new(world);

    // Let reflection framework know what kind of type this is
    ts.as_type(world.vector::<T>());

    // Forward std::vector value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &Vec<T>| {
        let world = unsafe { WorldRef::from_ptr(s.world as *mut sys::ecs_world_t) };
        let id = id!(world, T);
        for el in data.iter() {
            s.value_id(id, el as *const T as *const std::ffi::c_void);
        }
        0
    });

    // Return vector size
    ts.count(|data: &mut Vec<T>| data.len());

    fn ensure_generic_element<T: Default>(data: &mut Vec<T>, elem: usize) -> &mut T {
        if data.len() <= elem {
            data.resize_with(elem + 1, || T::default());
        }
        &mut data[elem]
    }

    fn resize_generic_vec<T: Default>(data: &mut Vec<T>, elem: usize) {
        data.resize_with(elem + 1, || T::default());
    }

    // Ensure element exists, return
    ts.ensure_element(ensure_generic_element::<T>);

    // Resize contents of vector
    ts.resize(resize_generic_vec::<T>);

    ts
}

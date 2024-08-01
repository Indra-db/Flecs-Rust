use core::ffi::CStr;
use flecs_ecs::prelude::*;
use flecs_ecs::sys;

macro_rules! generate_vec_meta_registration {
    ($world:ident, $($t:ty),*) => {
        $(
            {
                let id = id!($world, Vec<$t>);
                $world
                    .component_ext::<Vec<$t>>(id)
                    .opaque_func_id::<_, $t>(id, std_vector_support::<$t>);
            }
        )*
    };
}

pub(crate) fn meta_init_builtin(world: &World) {
    world.component::<String>().opaque_func(std_string_support);

    generate_vec_meta_registration!(
        world, String, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, bool, char,
        usize, isize
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
    ts.assign_string(|data: &mut String, value: *const i8| {
        *data = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() }
    });

    ts
}

pub fn std_vector_support<T: Default>(world: WorldRef) -> Opaque<Vec<T>, T> {
    let id = id!(&world, Vec<T>);
    let mut ts = Opaque::<Vec<T>, T>::new_id(world, id);

    // Let reflection framework know what kind of type this is
    ts.as_type(world.vector(id));

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

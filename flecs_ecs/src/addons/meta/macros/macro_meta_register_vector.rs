/// Registers a vector component type with metadata.
///
/// This macro expands to a function that registers a vector component type with metadata.
/// It performs the following actions:
/// - Generates a unique ID for the vector component type using the [`id!`](crate::addons::meta::id!) macro.
/// - Registers the vector component type with the [`World`](crate::core::World) using the [`component_ext`](crate::core::World::component_ext) function.
/// - Associates the generated function with the vector component type using the [`meta_register_vector_func`] macro.
///
/// # Parameters
/// - `$world`: The world reference.
/// - `$struct_type`: The struct type to be used for the vector component.
/// - `{ $($name:ident : $value:expr),* }`: the field data it should use when meta has to resize or ensure an element in the vector.
///
/// # Examples
///
/// public fields struct
/// ```
/// use flecs_ecs::prelude::*;
///
/// #[derive(Debug, Component)]
/// #[flecs(meta)]
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// let world = World::new();
///
/// world.component::<Point>().meta();
///
/// meta_register_vector_type!(&world, Point { x: 0.0, y: 0.0 });
///
/// //this then later on can be used like this...
/// let id = id!(&world, Vec<Point>);
/// let vec: Vec<Point> = vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
/// let json = world.to_json_dyn::<Vec<Point>>(id, &vec);
/// assert_eq!(json, "[{\"x\":1, \"y\":2}, {\"x\":3, \"y\":4}]");
/// ```
///
/// construction function
/// ```
/// use flecs_ecs::prelude::*;
///
/// #[derive(Debug, Component)]
/// #[flecs(meta)]
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// impl Point {
///     fn new(x: f32, y: f32) -> Self {
///         Self { x, y }
///     }
/// }
///
/// let world = World::new();
///
/// world.component::<Point>().meta();
///
/// meta_register_vector_type!(&world, Point::new(0.0, 0.0));
/// // if default is available, one can use that as well
/// // meta_register_vector_type!(&world, Point::default());
///
/// //this then later on can be used like this...
/// let id = id!(&world, Vec<Point>);
/// let vec: Vec<Point> = vec![Point::new(1.0, 2.0), Point::new(3.0, 4.0)];
/// let json = world.to_json_dyn::<Vec<Point>>(id, &vec);
/// assert_eq!(json, "[{\"x\":1, \"y\":2}, {\"x\":3, \"y\":4}]");
/// ```
#[macro_export]
macro_rules! meta_register_vector_type {
    ($world:expr, $struct_type:ident { $($name:ident : $value:expr),* $(,)? }) => {
        let id = id!($world, Vec<$struct_type>);
        $world
            .component_ext::<Vec<$struct_type>>(id)
            .opaque_func_id::<_, $struct_type>(id, meta_register_vector_func!($struct_type { $($name: $value),* }));
    };

    ($world:expr, $struct_type:ident :: $constructor:ident ( $($args:expr),* $(,)? )) => {
        let id = id!($world, Vec<$struct_type>);
        $world
            .component_ext::<Vec<$struct_type>>(id)
            .opaque_func_id::<_, $struct_type>(id, meta_register_vector_func!($struct_type::$constructor($($args),*)));
    };
}

/// Creates a function to manage a vector component type with metadata.
///
/// This macro expands to a function that manages a vector component type with metadata.
/// It performs the following actions:
/// - Initializes the vector component type using the specified field data.
/// - Registers serialization and deserialization functions for the vector component type.
/// - Defines resize and ensure element functions to handle vector resizing and element initialization.
///
/// # Parameters
/// - `$struct_type`: The struct type to be used for the vector component.
/// - `{ $($name:ident : $value:expr),* }`: The field data it should use when meta has to resize or ensure an element in the vector.
///
/// # Examples
///
/// public fields struct
/// ```
/// use flecs_ecs::prelude::*;
///
/// #[derive(Debug, Component)]
/// #[flecs(meta)]
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// let world = World::new();
///
/// world.component::<Point>().meta();
///
/// let id = id!(&world, Vec<Point>);
/// world
///     .component_ext::<Vec<Point>>(id)
///     .opaque_func_id::<_, Point>(id, meta_register_vector_func!(Point { x: 0.0, y: 0.0 }));
///
/// let vec: Vec<Point> = vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
/// let json = world.to_json_dyn::<Vec<Point>>(id, &vec);
/// assert_eq!(json, "[{\"x\":1, \"y\":2}, {\"x\":3, \"y\":4}]");
/// ```
///
/// // construction function
/// ```
/// use flecs_ecs::prelude::*;
///
/// #[derive(Debug, Component)]
/// #[flecs(meta)]
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// impl Point {
///     fn new(x: f32, y: f32) -> Self {
///         Self { x, y }
///     }
/// }
///
/// let world = World::new();
///
/// world.component::<Point>().meta();
///
/// let id = id!(&world, Vec<Point>);
/// world
///     .component_ext::<Vec<Point>>(id)
///     .opaque_func_id::<_, Point>(id, meta_register_vector_func!(Point::new(0.0, 0.0)));
///
/// let vec: Vec<Point> = vec![Point::new(1.0, 2.0), Point::new(3.0, 4.0)];
/// let json = world.to_json_dyn::<Vec<Point>>(id, &vec);
/// assert_eq!(json, "[{\"x\":1, \"y\":2}, {\"x\":3, \"y\":4}]");
/// ```
#[macro_export]
macro_rules! meta_register_vector_func {
    ($struct_type:ident { $($name:ident : $value:expr),* $(,)? }) => {
            |world: flecs_ecs::core::WorldRef| -> flecs_ecs::addons::meta::Opaque<Vec<$struct_type>, $struct_type> {
            let mut ts = flecs_ecs::addons::meta::Opaque::<Vec<$struct_type>, $struct_type>::new(world);

            // Let reflection framework know what kind of struct_type this is
            ts.as_type(world.vector::<$struct_type>());

            // Forward core::vector value to (JSON/...) serializer
            ts.serialize(|s: &flecs_ecs::addons::meta::Serializer, data: &Vec<$struct_type>| {
                let world = unsafe { WorldRef::from_ptr(s.world as *mut flecs_ecs::sys::ecs_world_t) };
                let id = id!(world, $struct_type);
                for el in data.iter() {
                    s.value_id(id, el as *const $struct_type as *const core::ffi::c_void);
                }
                0
            });

            // Return vector size
            ts.count(|data: &mut Vec<$struct_type>| data.len());

            fn ensure_element(data: &mut Vec<$struct_type>, elem: usize) -> &mut $struct_type {
                if data.len() <= elem {
                    data.resize_with(elem + 1, || $struct_type { $($name: $value),* });
                }
                &mut data[elem]
            }

            fn resize_vec(data: &mut Vec<$struct_type>, elem: usize) {
                data.resize_with(elem + 1, || $struct_type { $($name : $value),* });
            }

            // Ensure element exists, return
            ts.ensure_element(ensure_element);

            // Resize contents of vector
            ts.resize(resize_vec);

            ts
        }
    };
    // Match when using a construction function
    ($struct_type:ident :: $constructor:ident ( $($args:expr),* $(,)? )) => {
            |world: flecs_ecs::core::WorldRef| -> flecs_ecs::addons::meta::Opaque<Vec<$struct_type>, $struct_type> {
                let mut ts = flecs_ecs::addons::meta::Opaque::<Vec<$struct_type>, $struct_type>::new(world);

                // Let reflection framework know what kind of struct_type this is
                ts.as_type(world.vector::<$struct_type>());

                // Forward core::vector value to (JSON/...) serializer
                ts.serialize(|s: &flecs_ecs::addons::meta::Serializer, data: &Vec<$struct_type>| {
                    let world = unsafe { WorldRef::from_ptr(s.world as *mut flecs_ecs::sys::ecs_world_t) };
                    let id = id!(world, $struct_type);
                    for el in data.iter() {
                        s.value_id(id, el as *const $struct_type as *const core::ffi::c_void);
                    }
                    0
                });

                // Return vector size
                ts.count(|data: &mut Vec<$struct_type>| data.len());

                fn ensure_element(data: &mut Vec<$struct_type>, elem: usize) -> &mut $struct_type {
                    if data.len() <= elem {
                        data.resize_with(elem + 1, || $struct_type::$constructor($($args),*));
                    }
                    &mut data[elem]
                }

                fn resize_vec(data: &mut Vec<$struct_type>, elem: usize) {
                    data.resize_with(elem + 1, || $struct_type::$constructor($($args),*));
                }

                // Ensure element exists, return
                ts.ensure_element(ensure_element);

                // Resize contents of vector
                ts.resize(resize_vec);

                ts
            }
        };
}

pub use meta_register_vector_func;
pub use meta_register_vector_type;

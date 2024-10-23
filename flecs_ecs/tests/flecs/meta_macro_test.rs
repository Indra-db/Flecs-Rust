#![allow(clippy::float_cmp)]
use std::ffi::CStr;

use core::mem::offset_of;
use flecs_ecs::prelude::meta::*;
use flecs_ecs::{component, prelude::*};
use flecs_ecs_sys::ecs_world_t;

fn std_string_support(world: WorldRef) -> Opaque<String> {
    let mut ts = Opaque::<String>::new(world);

    // Let reflection framework know what kind of type this is
    ts.as_type(flecs::meta::String);

    // Forward std::string value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &String| {
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

fn std_vector_support<T: Default>(world: WorldRef) -> Opaque<Vec<T>, T> {
    let id = id!(&world, Vec<T>);
    let mut ts = Opaque::<Vec<T>, T>::new_id(world, id);

    // Let reflection framework know what kind of type this is
    ts.as_type(world.vector::<T>());

    // Forward std::vector value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &Vec<T>| {
        let world = unsafe { WorldRef::from_ptr(s.world as *mut ecs_world_t) };
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

#[test]
fn meta_struct() {
    let world = World::new();

    #[derive(Component)]
    struct Test {
        a: i32,
        b: f32,
    }

    let c = component!(&world, Test { a: i32, b: f32 });

    assert!(c.id() != 0);

    let a = c.lookup("a");
    assert!(a.id() != 0);
    assert!(a.has::<flecs::meta::Member>());

    a.get::<&flecs::meta::Member>(|mem| {
        assert_eq!(mem.type_, flecs::meta::I32);
    });

    let b = c.lookup("b");
    assert!(b.id() != 0);
    assert!(b.has::<flecs::meta::Member>());

    b.get::<&flecs::meta::Member>(|mem| {
        assert_eq!(mem.type_, flecs::meta::F32);
    });
}

#[test]
fn meta_nested_struct() {
    let world = World::new();

    #[derive(Component)]
    struct Test {
        x: i32,
    }

    #[derive(Component)]
    struct Nested {
        a: Test,
    }

    let t = component!(&world, Test { x: i32 });

    let n = component!(&world, Nested { a: Test });

    assert!(n.id() != 0);

    let a = n.lookup("a");
    assert!(a.id() != 0);
    assert!(a.has::<flecs::meta::Member>());

    a.get::<&flecs::meta::Member>(|mem| {
        assert_eq!(mem.type_, t.id());
    });
}

#[test]
fn meta_struct_w_portable_type() {
    let world = World::new();

    #[derive(Component)]
    struct Test {
        a: usize,
        b: usize,
        c: Entity,
        d: Entity,
    }

    let t = component!(
        &world,
        Test {
            a: usize,
            b: usize,
            c: Entity,
            d: Entity
        }
    );

    assert!(t.id() != 0);

    let a = t.lookup("a");
    assert!(a.id() != 0);
    assert!(a.has::<flecs::meta::Member>());

    a.get::<&flecs::meta::Member>(|mem| {
        assert_eq!(mem.type_, flecs::meta::UPtr);
    });

    let b = t.lookup("b");
    assert!(b.id() != 0);
    assert!(b.has::<flecs::meta::Member>());

    // b.get::<&flecs::meta::Member>(|mem| {
    //     assert_eq!(mem.type_, flecs::meta::UPtr);
    // });

    // let c = t.lookup("c");
    // assert!(c.id() != 0);
    // assert!(c.has::<flecs::meta::Member>());

    // c.get::<&flecs::meta::Member>(|mem| {
    //     assert_eq!(mem.type_, flecs::meta::Entity);
    // });

    // let d = t.lookup("d");
    // assert!(d.id() != 0);
    // assert!(d.has::<flecs::meta::Member>());

    // d.get::<&flecs::meta::Member>(|mem| {
    //     assert_eq!(mem.type_, flecs::meta::Entity);
    // });
}

//TODO meta_units -- units addon is not yet implemented in Rust
//TODO Meta_unit_w_quantity -- units addon is not yet implemented in Rust
//TODO Meta_unit_w_prefix -- units addon is not yet implemented in Rust
//TODO Meta_unit_w_over -- units addon is not yet implemented in Rust

#[test]
fn meta_partial_struct() {
    let world = World::new();

    #[derive(Component)]
    struct Position {
        x: f32,
    }

    let c = component!(&world, Position { x: f32 });

    assert!(c.id() != 0);

    c.get::<&flecs::Component>(|ptr| {
        assert_eq!(ptr.size, 4);
        assert_eq!(ptr.alignment, 4);
    });

    let xe = c.lookup("x");
    assert!(xe.id() != 0);
    assert!(xe.has::<flecs::meta::Member>());
    xe.get::<&flecs::meta::Member>(|x| {
        assert_eq!(x.type_, flecs::meta::F32);
        assert_eq!(x.offset, 0);
    });
}

#[test]
fn meta_partial_struct_custom_offset() {
    let world = World::new();

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    let c = component!(&world, Position { y: f32 });

    assert!(c.id() != 0);

    c.get::<&flecs::Component>(|ptr| {
        assert_eq!(ptr.size, 8);
        assert_eq!(ptr.alignment, 4);
    });

    let xe = c.lookup("y");
    assert!(xe.id() != 0);
    assert!(xe.has::<flecs::meta::Member>());
    xe.get::<&flecs::meta::Member>(|x| {
        assert_eq!(x.type_, flecs::meta::F32);
        assert_eq!(x.offset, 4);
    });
}

#[test]
fn meta_bitmask() {
    let world = World::new();

    #[derive(Component)]
    struct Toppings {
        value: u32,
    }

    impl Toppings {
        const BACON: u32 = 0x1;
        const LETTUCE: u32 = 0x2;
        const TOMATO: u32 = 0x4;

        fn new() -> Self {
            Toppings { value: 0 }
        }

        fn add(&mut self, topping: u32) {
            self.value |= topping;
        }

        fn remove(&mut self, topping: u32) {
            self.value &= !topping;
        }

        fn has(&self, topping: u32) -> bool {
            self.value & topping != 0
        }
    }

    #[derive(Component)]
    struct Sandwich {
        toppings: Toppings,
    }

    impl Sandwich {
        fn new() -> Self {
            Sandwich {
                toppings: Toppings::new(),
            }
        }

        fn add_topping(&mut self, topping: u32) {
            self.toppings.add(topping);
        }

        fn remove_topping(&mut self, topping: u32) {
            self.toppings.remove(topping);
        }

        fn has_topping(&self, topping: u32) -> bool {
            self.toppings.has(topping)
        }
    }

    world
        .component::<Toppings>()
        .bit("bacon", Toppings::BACON)
        .bit("lettuce", Toppings::LETTUCE)
        .bit("tomato", Toppings::TOMATO);

    component!(&world, Sandwich { toppings: Toppings });

    // Create entity with Sandwich as usual
    let e = world.entity().set(Sandwich {
        toppings: Toppings {
            value: Toppings::BACON | Toppings::LETTUCE,
        },
    });

    // Convert Sandwidth component to flecs expression string
    e.get::<&Sandwich>(|val| {
        assert_eq!(world.to_expr(val), "{toppings: lettuce|bacon}");
    });
}

#[test]
fn meta_world_ser_deser_flecs_entity() {
    #[derive(Component)]
    struct RustEntity {
        entity: Entity,
    }

    let world = World::new();

    component!(&world, RustEntity { entity: Entity });

    let e1 = world.entity_named("ent1");
    let e2 = world
        .entity_named("ent2")
        .set(RustEntity { entity: e1.id() });

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });

    let json = world.to_json_world(None);

    let world = World::new();

    component!(&world, RustEntity { entity: Entity });

    world.from_json_world(json.as_str(), None);

    assert!(e1.is_alive());
    assert!(e2.is_alive());

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });
}

#[test]
fn meta_new_world_ser_deser_flecs_entity() {
    #[derive(Component)]
    struct RustEntity {
        entity: Entity,
    }

    let world = World::new();

    component!(&world, RustEntity { entity: Entity });

    let e1 = world.entity_named("ent1");
    let e2 = world
        .entity_named("ent2")
        .set(RustEntity { entity: e1.id() });

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });

    let json = world.to_json_world(None);

    let world = World::new();

    component!(&world, RustEntity { entity: Entity });

    world.from_json_world(json.as_str(), None);

    let e1 = world.lookup("ent1");
    let e2 = world.lookup("ent2");

    assert!(e1.id() != 0);
    assert!(e2.id() != 0);

    assert!(e1.is_alive());
    assert!(e2.is_alive());

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });
}

#[test]
fn meta_new_world_ser_deser_empty_flecs_entity() {
    #[derive(Component)]
    struct RustEntity {
        entity: Entity,
    }

    let world = World::new();

    component!(&world, RustEntity { entity: Entity });

    let e1 = Entity::null();
    let e2 = world.entity_named("ent2").set(RustEntity { entity: e1 });

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"#0\"}");
    });

    let json = world.to_json_world(None);

    let world = World::new();

    component!(&world, RustEntity { entity: Entity });

    world.from_json_world(json.as_str(), None);

    let e2 = world.lookup("ent2");

    assert!(e2.id() != 0);

    assert!(e2.is_alive());

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"#0\"}");
    });
}

#[repr(C)]
#[derive(Component)]
#[allow(clippy::enum_clike_unportable_variant)]
enum EnumWithBits {
    BitA = 0,
    BitB = 1 << 0,
    BitAll = 0xffffffff,
}

#[derive(Component)]
struct EnumWithBitsStruct {
    bits: EnumWithBits,
}

impl Default for EnumWithBitsStruct {
    fn default() -> Self {
        EnumWithBitsStruct {
            bits: EnumWithBits::BitAll,
        }
    }
}

#[test]
fn meta_struct_member_ptr() {
    let world = World::new();

    #[derive(Component)]
    struct Test {
        x: i32,
    }

    #[derive(Component)]
    struct Test2 {
        y: f64,
    }

    #[derive(Component)]
    struct Nested {
        a: Test, //what offset should this be? Rust returns 16, but CPP gives 0 lol
        pad: i32,
        b: [Test2; 2],
    }

    let t = component!(&world, Test { x: i32 });

    let t2 = component!(&world, Test2 { y: f64 });

    let n = component!(
        &world,
        Nested {
            a: Test,
            b: [Test2; 2]
        }
    );

    //validate Test #1
    assert!(t.id() != 0);

    let x = t.lookup("x");
    assert!(x.id() != 0);
    assert!(x.has::<flecs::meta::Member>());
    x.get::<&flecs::meta::Member>(|xm| {
        assert_eq!(xm.type_, flecs::meta::I32);
        assert_eq!(xm.offset, offset_of!(Test, x) as i32);
    });

    //validate Test2 #2
    assert!(t2.id() != 0);

    let y = t2.lookup("y");
    assert!(y.id() != 0);
    assert!(y.has::<flecs::meta::Member>());
    y.get::<&flecs::meta::Member>(|ym| {
        assert_eq!(ym.type_, flecs::meta::F64);
        assert_eq!(ym.offset, offset_of!(Test2, y) as i32);
    });

    // Validate Nested
    assert!(n.id() != 0);

    let a = n.lookup("a");
    assert!(a.id() != 0);
    assert!(a.has::<flecs::meta::Member>());
    a.get::<&flecs::meta::Member>(|am| {
        assert_eq!(am.type_, t.id());
        let offset = offset_of!(Nested, a) as i32;
        assert_eq!(am.offset, offset);
    });

    let b = n.lookup("b");
    assert!(b.id() != 0);
    assert!(b.has::<flecs::meta::Member>());
    b.get::<&flecs::meta::Member>(|bm| {
        assert_eq!(bm.type_, t2.id());
        assert_eq!(bm.offset, offset_of!(Nested, b) as i32);
        assert_eq!(bm.count, 2);
    });
}

#[test]
fn meta_struct_field_order() {
    let world = World::new();

    #[derive(Component)]
    struct Test {
        a: u32,
        b: i32,
    }

    let t = component!(&world, Test { a: u32, b: i32 });

    assert_ne!(t.id(), 0);
    let a = t.lookup("a");
    assert_ne!(a.id(), 0);
    assert!(a.has::<flecs::meta::Member>());
    a.get::<&flecs::meta::Member>(|am| {
        assert_eq!(am.type_, flecs::meta::U32);
        assert_eq!(am.offset, offset_of!(Test, a) as i32);
    });

    assert_ne!(t.id(), 0);
    let b = t.lookup("b");
    assert_ne!(b.id(), 0);
    assert!(b.has::<flecs::meta::Member>());
    b.get::<&flecs::meta::Member>(|bm| {
        assert_eq!(bm.type_, flecs::meta::I32);
        assert_eq!(bm.offset, offset_of!(Test, b) as i32);
    });

    let e = world.entity().set(Test { a: 10, b: 20 });

    e.get::<&Test>(|ptr| {
        assert_eq!(ptr.a, 10);
        assert_eq!(ptr.b, 20);
        let json = world.to_expr(ptr);
        assert_eq!(json, "{a: 10, b: 20}"); //if this fails, field re-ordering is not working
    });
}

#[test]
fn meta_ser_deser_option() {
    let world = World::new();

    #[derive(Component, Default, Debug, PartialEq)]
    struct OptComponent(Option<u32>);

    component!(&world, #[auto] Option<u32>);
    component!(&world, OptComponent(Option<u32>));

    {
        let mut v = OptComponent::default();
        let json = "{\"0\":{\"None\":false}}".to_string();
        world.from_json::<OptComponent>(&mut v, &json, None);
        assert_eq!(v, OptComponent(None));
        let json = world.to_json::<OptComponent>(&v);
        assert_eq!(json, "{\"0\":{\"None\":false}}");
    }

    {
        let mut v = OptComponent::default();
        let json = "{\"0\":{\"Some\":42}}".to_string();
        world.from_json::<OptComponent>(&mut v, &json, None);
        assert_eq!(v, OptComponent(Some(42)));
        let json = world.to_json::<OptComponent>(&v);
        assert_eq!(json, "{\"0\":{\"Some\":42}}");
    }
}

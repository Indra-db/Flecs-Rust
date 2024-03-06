#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(warnings)]
#![allow(unused_macros)]

use flecs_ecs::addons::app::App;

use flecs_ecs::{
    addons::system::SystemBuilder,
    core::{
        builder::Builder,
        c_types::{EntityT, IdT, WorldT, ECS_ON_ADD},
    },
    sys::{self, ecs_filter_desc_t},
};

use flecs_ecs::{
    core::{
        component_registration::*,
        entity::Entity,
        entity_view::EntityView,
        enum_type::CachedEnumData,
        event::{EventBuilderImpl, EventData},
        event_builder::EventBuilderTyped,
        filter::Filter,
        filter_builder::{FilterBuilder, FilterBuilderImpl, FilterType},
        iterable::Iterable,
        observer_builder::{ObserverBuilder, ObserverBuilderImpl},
        query::Query,
        query_builder::QueryBuilder,
        term::TermBuilder,
        world::World,
        FlecsErrorCode,
    },
    ecs_abort, ecs_assert,
};
use flecs_ecs_derive::Component;
use rand::{seq::index, Rng};
use seq_macro::seq;
use std::{ffi::CStr, fmt::Display, sync::OnceLock};
//use flecs_ecs_derive::print_foo;

//#[macro_use]
//extern crate debug_here;

#[derive(Clone, Default, Debug, Component)]
struct TypeB {
    x: i32,
    v: Vec<i32>,
}

impl Drop for TypeB {
    fn drop(&mut self) {
        println!("destructor - de-allocating vector");
    }
}
#[derive(Clone, Default, Debug, Component)]
struct TypeAc {
    x: i32,
    y: i32,
}

#[derive(Clone, Default, Debug, Component)]
struct TypeA {
    x: i32,
    y: i32,
}
#[derive(Clone, Default, Debug, Component)]
struct TypeD {
    name: String,
}

impl Drop for TypeD {
    fn drop(&mut self) {
        println!("destructor - typeD");
    }
}
#[derive(Clone, Component, Debug)]
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

#[derive(Clone, Debug, Component)]
enum Shape {
    Circle(f64),             // Contains a single f64 representing radius
    Rectangle(f64, f64),     // Contains two f64 values representing width and height
    Triangle(f64, f64, f64), // Contains three f64 values representing the lengths of the sides
}
#[derive(Clone, Component, Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 }, // Contains named data 'x' and 'y' of type i32
    Write(String),           // Contains a single unnamed String
    ChangeColor { r: i8, g: i8, b: i8 }, // Contains named data 'r', 'g', and 'b' of type i8
}

impl Default for Shape {
    fn default() -> Self {
        Self::Circle(10.0)
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::Quit
    }
}

impl Default for TrafficLight {
    fn default() -> Self {
        Self::Red
    }
}

#[derive(Clone, Component, Debug)]
pub enum Movement {
    Walking,
    Running,
}

impl Default for Movement {
    fn default() -> Self {
        Self::Walking
    }
}

#[derive(Clone, Debug, Component)]
enum EnumTest {
    TypeX(TypeA),
    TypeB,
    TypeD,
}

impl Default for EnumTest {
    fn default() -> Self {
        Self::TypeD
    }
}

#[derive(Clone, Default, Debug, Component)]
struct FTest {}

#[derive(Clone, Default, Debug, Component)]
struct Test2 {}

#[derive(Clone, Default, Debug, Component)]
struct EntityTest {}

impl EntityTest {
    fn new() -> Self {
        Self {}
    }

    #[allow(clippy::extra_unused_type_parameters)]
    pub fn is_enum_component<T: CachedComponentData + ComponentType<Enum>>(&self) {
        println!("is_enum_component");
    }

    #[allow(clippy::extra_unused_type_parameters)]
    pub fn is_struct_component<T: CachedComponentData + ComponentType<Struct>>(&self) {
        println!("is_struct_component");
    }

    #[allow(clippy::extra_unused_type_parameters)]
    pub fn is_empty_component<T: CachedComponentData + EmptyComponent>(&self) {
        println!("is_empty_component");
    }

    #[allow(clippy::extra_unused_type_parameters)]
    pub fn is_not_empty_component<T: CachedComponentData + NotEmptyComponent>(&self) {
        println!("is_not_empty_component");
    }
}

pub fn get_index(mov: Movement) -> i32 {
    match mov {
        Movement::Walking => 0,
        Movement::Running => 1,
    }
}

static mut MY_ARRAY: [i32; 12] = [0; 12];

// Your original traits
trait TraitY {
    fn function_from_trait_y(&self);
}

trait TraitFoo {
    //type Marker;
    fn bar(&self);
}

// Marker structs to differentiate implementations
struct HasTraitY;
struct NoTraitY;

// Implementations of TraitFoo based on whether TraitY is implemented
impl TraitFoo for ImplementsTraitY {
    //type Marker = HasTraitY;
    fn bar(&self) {
        function_b(self);
    }
}

impl TraitFoo for DoesNotImplementTraitY {
    //type Marker = NoTraitY;
    fn bar(&self) {
        function_a();
    }
}

// Assume function_a() and function_b() are defined somewhere
fn function_a() {
    println!("Called function A");
}

fn function_b<T: TraitY>(_t: &T) {
    println!("Called function B");
    _t.function_from_trait_y();
}

// Some structs to test with
struct ImplementsTraitY;
impl TraitY for ImplementsTraitY {
    fn function_from_trait_y(&self) {
        println!("Called function from TraitY");
    }
}

struct DoesNotImplementTraitY;

#[derive(Debug, Clone, Default, Component)]
struct CompA {}

#[derive(Debug, Clone, Default, Component)]
struct CompB {}
fn testp() {
    let world = World::new();
    let entity = Entity::new(&world);

    entity.add::<CompA>().add_pair::<CompB, TypeA>();
}

/// ```
///  struct Position{}
/// ```
///

#[derive(Debug, Default, Component, Clone)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, Component, Clone)]
pub struct Vel {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, Component, Clone)]
pub struct Rot {
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Default, Component, Clone)]
pub struct Mass {
    pub x: f32,
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl Display for Vel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl Display for Rot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

seq!(P in 0..=20 {
    // expands to structs named x0, x1, x2, ..., 20
    #[derive(Debug, Default, Clone, Component)]
    struct X~P
    {
        x: f32,
        y: f32,
    }
});

///////////////////////////////////////////////
///////////////////////////////////////////////
///////////////////////////////////////////////
///
// Assuming these are the enum definitions in Rust

fn flip_coin() -> bool {
    rand::random::<bool>()
}

impl EventData for X10 {}

struct A {
    value: i32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Component)]
enum Color {
    Green,
    #[default]
    Red,
    Blue,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Component)]
enum Colorx {
    Green,
    #[default]
    Red,
    Blue,
}
trait X {
    fn x(&self) -> String;
}
trait Y {
    fn y(&self) -> String;
}

trait Handle<T: ?Sized> {
    fn handle();
}
impl<T: X> Handle<dyn X> for T {
    fn handle() {
        println!("X");
    }
}
impl<T: Y> Handle<dyn Y> for T {
    fn handle() {
        println!("Y");
    }
}

pub fn get<T: Handle<P>, P: ?Sized>() {
    T::handle();
}

pub struct TestX {}
impl X for TestX {
    fn x(&self) -> String {
        "X".to_string()
    }
}

pub struct TestY {}
impl Y for TestY {
    fn y(&self) -> String {
        "Y".to_string()
    }
}

fn main() {
    // get::<TestX, _>();
    // get::<TestY, _>();
    //static mut ENUM_FIELD_ENTITY_ID: [u64; 3] = [0; 3];
    //let ptr = unsafe { ENUM_FIELD_ENTITY_ID.as_mut_ptr() };
    //println!("ptr: {:?}", ptr);
    //
    //let ptr2 = Color::__get_enum_data_ptr_mut();
    //println!("ptr2: {:?}", ptr2);
    //let val = unsafe { Color::get_entity_id_from_enum_field_index(1) };
    //println!("val: {:?}", val);

    //let test = copy_and_allocate_c_char_from_rust("test");
    ///unsafe {
    ///    ecs_os_set_api_defaults();
    ///}
    let world = World::new();

    let mut system2 = SystemBuilder::<(&mut Pos, &Vel)>::new_named(
        &world,
        CStr::from_bytes_with_nul(b"x\0").unwrap(),
    )
    .kind_id(flecs_ecs::core::ECS_ON_START)
    .on_each_entity(|entity, (pos, vel)| {
        println!("startup");
        println!("entity: {:?}", entity.get_name());
    });

    let mut system = SystemBuilder::<(&mut Pos, &Vel)>::new_named(
        &world,
        CStr::from_bytes_with_nul(b"xx\0").unwrap(),
    )
    .kind_id(flecs_ecs::core::ECS_ON_UPDATE)
    .on_each_entity(|entity, (pos, vel)| {
        println!("update");
        println!("entity: {:?}", entity.get_name());
    });

    let e1 = world
        .new_entity_named(CStr::from_bytes_with_nul(b"entity1\0").unwrap())
        .set(Pos { x: 10.0, y: 20.0 })
        .set(Vel { x: 1.0, y: 2.0 });

    let e2 = world
        .new_entity_named(CStr::from_bytes_with_nul(b"entity2\0").unwrap())
        .set(Pos { x: 10.0, y: 20.0 })
        .set(Vel { x: 3.0, y: 4.0 });

    let e3 = world
        .new_entity_named(CStr::from_bytes_with_nul(b"entity3\0").unwrap())
        .set(Pos { x: 10.0, y: 20.0 });

    world.progress();
    world.progress();

    let ss = flecs_ecs::core::get_only_type_name::<Pos>();

    // Run the system
    //system.run(0.0, std::ptr::null_mut());
    //let mut count = 0;
    //let count_ptr = &mut count as *mut i32;
    //
    //let observer = ObserverBuilder::<(Pos, Vel)>::new(&world)
    //    .add_event(ECS_ON_ADD)
    //    .on_iter_only(move |entity| {
    //        unsafe {
    //            // Unsafe mutable access inside the closure
    //            *count_ptr += 1;
    //            println!("count0: {:?}", *count_ptr);
    //        }
    //    });
    //
    //let entity = world.new_entity();
    //entity.set(Pos { x: 10.0, y: 20.0 });
    //entity.set(Vel { x: 1.0, y: 2.0 });
    //println!("entity id: {:?}", entity.id.raw_id);
    //
    //let entity = world.new_entity();
    //entity.set(Pos { x: 10.0, y: 20.0 });
    //entity.set(Vel { x: 1.0, y: 2.0 });
    //println!("entity id: {:?}", entity.id.raw_id);
    //
    //for _ in 0..100 {
    //    let mut e = world.new_entity();
    //    e.add::<Pos>();
    //    e.set(Vel { x: 5.0, y: 5.0 });
    //    if flip_coin() {
    //        e.add::<X2>();
    //    }
    //    if flip_coin() {
    //        e.add::<X3>();
    //    }
    //    if flip_coin() {
    //        e.add::<X4>();
    //    }
    //    if flip_coin() {
    //        e.add::<X5>();
    //    }
    //    if flip_coin() {
    //        e.add::<X6>();
    //    }
    //    if flip_coin() {
    //        e.add::<X7>();
    //    }
    //    if flip_coin() {
    //        e.add::<X8>();
    //    }
    //    if flip_coin() {
    //        e.add::<X9>();
    //    }
    //    if flip_coin() {
    //        e.add::<X10>();
    //    }
    //    if flip_coin() {
    //        e.add::<X11>();
    //    }
    //}
    //
    //let query_builder = QueryBuilder::<(Pos, Vel)>::new(&world)
    //    .term_at(2)
    //    .parent()
    //    .build()
    //    .each(|(pos, vel)| {
    //        println!("pos: {:?}", pos);
    //        pos.x += vel.x;
    //        pos.y += vel.y;
    //    });

    ///////let mut eventb: EventBuilderTyped<X10> = EventBuilderTyped::<X10>::new(&world, 0);
    ///////let eventdata: X10 = X10 { x: 10.0, y: 20.0 };
    ///////eventb.set_event_data(&eventdata);
    ///////eventb.emit();

    //test ecs_assert
    ///////ecs_assert!(true, "This is a test");
    //e.set(Rot { x: 10.0, y: 30.0 });

    //let mut v: Vec<f32> = vec![0.0; 1000500];
    //let mut query = Query::<(Option<Pos>,)>::new(&world);
    //
    //query.each(|(posÍ)| {});
    //query.each_entity(|entity, (pos)| {});
    //query.iter(|it, (pos)| {});
    //query.iter_only(|it| {});

    //let mut index: usize = 0;
    //pp let mut query2 = Query::<(Pos, Vel)>::new(&world);
    //pp //for i in 0..1001 {
    //pp //    query2.iter(|it, (pos, vel)| {
    //pp //        for (p, v) in pos.iter_mut().zip(vel.iter()) {
    //pp //            p.x += v.x;
    //pp //            p.y += v.y;
    //pp //        }
    //pp //    });
    //pp //}
    //pp
    //pp for i in 0..1001 {
    //pp     query2.iter_only(|it| {
    //pp         let mut pos = it.get_field_data::<Pos>(1);
    //pp         let mut vel = it.get_field_data::<Vel>(2);
    //pp
    //pp         for index in 0..it.count() {
    //pp             let mut p = &mut pos[index];
    //pp             let mut v = &mut vel[index];
    //pp             p.x += v.x;
    //pp             p.y += v.y;
    //pp         }
    //pp     });
    //pp }
    //pp
    //pp let start2 = std::time::Instant::now();
    //pp std::hint::black_box(for i in 0..15001 {
    //pp     query2.iter_only(|it| {
    //pp         let mut pos = it.get_field_data::<Pos>(1);
    //pp         let mut vel = it.get_field_data::<Vel>(2);
    //pp
    //pp         for index in 0..it.count() {
    //pp             let mut p = &mut pos[index];
    //pp             let mut v = &mut vel[index];
    //pp             p.x += v.x;
    //pp             p.y += v.y;
    //pp         }
    //pp     });
    //pp });
    //pp let duration2 = start2.elapsed();

    // /let start = std::time::Instant::now();
    // /std::hint::black_box(for i in 0..15001 {
    // /    query.each(|(pos, vel)| {
    // /        pos.x += vel.x;
    // /        pos.y += vel.y;
    // /    });
    // /});
    // /let duration = start.elapsed();
    // /
    // /println!("Time elapsed : {:?}", duration);
    // /println!("Time elapsed per query: {:?}", duration / 15000);
    //pp println!("Time elapsed : {:?}", duration2);
    //pp println!("Time elapsed per query: {:?}", duration2 / 15000);

    ///////println!("starting");
    ///////
    ///////App::new(&world).enable_rest(0).run();
    ///////println!("ending");
    //println!("index: {:?}", index);
    //
    //let mut archetype_count = 0;
    //query.iter(|it| {
    //    archetype_count += 1;
    //    //let positions = it.field::<Pos>(1);
    //    //let vels = it.field::<Vel>(2);
    //    //
    //    //for index in 0..it.count() {
    //    //    let pos = positions.get(index);
    //    //    let vel = vels.get(index);
    //    //    println!("Iter - {:?}, {:?}", pos, vel);
    //    //}
    //});
    //
    //println!("archetype_count: {:?}", archetype_count);
    ////ttt let world = World::default();
    ////ttt let mut tag_to_find = 0;
    ////ttt for i in 0..1000 {
    ////ttt     let tag = world.new_entity();
    ////ttt     if i == 500 {
    ////ttt         tag_to_find = tag.raw_id;
    ////ttt     }
    ////ttt     for _j in 0..100 {
    ////ttt         let entity = world
    ////ttt             .new_entity()
    ////ttt             .set(Pos { x: 10.0, y: 20.0 })
    ////ttt             .set(Vel { x: 10.0, y: 20.0 })
    ////ttt             .set(Rot { x: 10.0, y: 20.0 })
    ////ttt             .add_id(tag.raw_id);
    ////ttt     }
    ////ttt }
    ////ttt
    ////ttt let mut query = QueryBuilder::<(Pos, Vel)>::new(&world)
    ////ttt     .with(With::Id((tag_to_find)))
    ////ttt     .build();
    ////ttt
    ////ttt let mut counter = 0;
    ////ttt let start = std::time::Instant::now();
    ////ttt
    ////ttt query.each(|(pos, vel)| {
    ////ttt     counter += 1;
    ////ttt     pos.x += vel.x;
    ////ttt     pos.y += vel.y;
    ////ttt });
    ////ttt
    ////ttt let duration = start.elapsed();
    ////ttt println!("Time elapsed : {:?}", duration);
    ////ttt println!("counter: {:?}", counter);
    //
    ////std::thread::sleep(std::time::Duration::from_secs(2));
    ////query.each(|(pos, vel)| {
    ////    println!("pos: {:?}", pos);
    ////});

    //let filter_builder = FilterBuilder::new()
    //std::env::set_var("RUST_BACKTRACE", "1");
    //let world = World::default();

    //let oper = type_to_oper::<*const i32>();
    //println!("{:?}", oper); // Should print "Optional"

    //let tag_a = world.new_entity();

    //for _i in 0..10000000 {
    //    let entity = world
    //        .new_entity()
    //        .add::<Pos>()
    //        .add::<Vel>()
    //        .set(Rot { x: 10.0, y: 20.0 });
    //}

    //xx for _i in 0..1000 {
    //xx     let tag = world.new_entity();
    //xx     for _j in 0..10000 {
    //xx         let entity = world
    //xx             .new_entity()
    //xx             .set(Pos { x: 10.0, y: 20.0 })
    //xx             .set(Vel { x: 10.0, y: 20.0 })
    //xx             .set(Rot { x: 10.0, y: 20.0 })
    //xx             .add_id(tag.raw_id);
    //xx     }
    //xx }

    //let entity = world.new_entity().add::<Pos>();
    //
    //let tag_a_entity = world
    //    .new_entity()
    //    .add_id(tag_a.raw_id)
    //    .add::<Pos>()
    //    .add::<Vel>();
    //
    //let tag_a_entity = world
    //    .new_entity()
    //    .add_id(tag_a.raw_id)
    //    .add::<Pos>()
    //    .add::<Vel>();

    //for _i in 0..1000 {
    //    let tag = world.new_entity();
    //    for _j in 0..1 {
    //        let entity = world
    //            .new_entity()
    //            .set(Pos { x: 10.0, y: 20.0 })
    //            .set(Vel { x: 10.0, y: 20.0 })
    //            .add_id(tag.raw_id);
    //    }
    //}
    //let start = std::time::Instant::now();
    //for _i in 0..1000000 {
    //    for _j in 0..300 {
    //        let entity = world.new_entity();
    //        entity.destruct();
    //    }
    //}

    //let duration = start.elapsed();
    //println!("Time elapsed : {:?}", duration);

    //xx let mut filter = Query::<(Pos, Vel, Rot)>::new(&world);

    //parent let parent = world.new_entity().set(Pos { x: 5.0, y: 5.0 });
    //parent let child = world
    //parent     .new_entity()
    //parent     .child_of_id(parent.raw_id)
    //parent     .set(Pos { x: 10.0, y: 20.0 })
    //parent     .add::<Vel>()
    //parent     .add::<Rot>();
    //parent
    //parent let child2 = world
    //parent     .new_entity()
    //parent     .child_of_id(parent.raw_id)
    //parent     .set(Pos { x: 10.0, y: 20.0 })
    //parent     .add::<Vel>();
    //parent
    //parent let child3 = world
    //parent     .new_entity()
    //parent     .child_of_id(parent.raw_id)
    //parent     .set(Pos { x: 10.0, y: 20.0 })
    //parent     .add::<Vel>()
    //parent     .add::<Rot>();
    //parent
    //parent let child3 = world.new_entity().set(Pos { x: 10.0, y: 20.0 });
    //parent
    //parent let mut filter = FilterBuilder::<(Pos, Pos, Vel, Option<Rot>)>::new(world.raw_world)
    //parent     .term_at(2)
    //parent     .parent()
    //parent     .instanced()
    //parent     .build();

    //let mut filter = Query::<(Pos, Vel, Rot)>::new(&world);
    //  =let mut filter: Filter<_>
    //let mut filter: Filter<_> = FilterBuilder::<(Pos, Option<Vel>)>::new(world.raw_world)
    //    .without(Without::Id(tag_a.raw_id))
    //    .build();
    //.build();

    //parent filter.each(|(pos, pos_parent, vel, rot)| {
    //parent     counter += 1;
    //parent     println!("pos: {:?}", pos);
    //parent     println!("pos_parent: {:?}", pos_parent);
    //parent     pos.x += pos_parent.x;
    //parent     println!("pos: {:?}", pos);
    //parent     println!("---")
    //parent     //println!("entity: {:?}", e);
    //parent });

    //xx let mut counter = 0;
    //xx let start = std::time::Instant::now();
    //xx
    //xx filter.each(|(pos, vel, rot)| {
    //xx     counter += 1;
    //xx     pos.x += vel.x;
    //xx     pos.y += vel.y;
    //xx });
    //xx
    //xx let duration = start.elapsed();
    //xx println!("Time elapsed : {:?}", duration);
    //xx println!("counter: {:?}", counter);

    //// let e1 = world
    ////     .new_entity()
    ////     .add::<Pos>()
    ////     .add::<Vel>();
    ////
    //// let e2 = world.new_entity().add::<Vel>();
    ////
    //// let mut filter = Filter::<(Vel, Option<Pos>)>::new(&world);
    //// let mut counter = 0;
    //// let start = std::time::Instant::now();
    ////
    //// filter.each(|(pos, vel)| {
    ////     counter += 1;
    ////     println!("pos: {:?}", pos);
    //// });
    ////
    //// let duration = start.elapsed();
    //// println!("Time elapsed : {:?}", duration);
    //// println!("counter: {:?}", counter);

    //println!("counter: {:?}", counter);
    //let mut input = String::new();
    //std::io::stdin()
    //    .read_line(&mut input)
    //    .expect("Failed to read line");
    //let start = std::time::Instant::now();
    //
    //filter.each_entity(|_, (pos, vel)| {
    //    pos.x += vel.x;
    //    pos.y += vel.y;
    //});
    //
    //let duration = start.elapsed();
    //println!("Time elapsed with entity: {:?}", duration);

    //let world = World::default();
    //let entity = world.new_entity();
    //entity.enable_component::<TypeA>();
    //let is_enabled = entity.is_component_enabled::<TypeA>();
    //println!("is_enabled: {:?}", is_enabled); //true
    //println!("has: {:?}", entity.has::<TypeA>()); //false

    //let mut world = World::default();
    //let mut entity = Entity::new(world.raw_world);
    //
    ////world = world.add::<TypeD>();
    ////let compa = world.get::<TypeD>();
    //
    ////println!("compa: {:?}", compa);
    ////println!(" compa id: {:?}", TypeD::get_id(world.raw_world));
    //
    //entity = entity.set(TypeD {
    //    name: "test".to_string(),
    //});
    //
    //world.defer_begin();
    //entity = entity.add::<CompA>();
    //entity = entity.add::<CompB>();
    //entity = entity.add::<TypeA>();
    //world.defer_end();
    //println!("cx");

    //entity = entity.add::<TypeD>();

    //std::mem::forget(typed);
    //println!("entity: {:?}", entity);
    //println!("entity: {:?}", entity.get::<TypeA>());
    //println!("entity: {:?}", entity.get::<TypeD>());
}

/*
fn main() {
    //for enum_type in Movement::iter() {
    //    let str_name = enum_type.get_cstr_name();
    //    let index = enum_type.get_enum_index();
    //    println!("enum_type: {:?}", str_name);
    //    println!("enum_type: {:?}", enum_type);
    //    println!("index: {:?}", index);
    //}
    //println!("field size movement {:?}", Movement::SIZE_ENUM_FIELDS);
    //println!();
    //for enum_type in EnumTest::iter() {
    //    println!("enum_type: {:?}", enum_type);
    //}
    //println!("field size movement {:?}", EnumTest::SIZE_ENUM_FIELDS);
    //println!();
    //for enum_type in Shape::iter() {
    //    println!("enum_type: {:?}", enum_type);
    //}
    //println!("field size movement {:?}", Shape::SIZE_ENUM_FIELDS);
    //println!();
    //for enum_type in Message::iter() {
    //    println!("enum_type: {:?}", enum_type);
    //}
    //println!("field size movement {:?}", Message::SIZE_ENUM_FIELDS);
    //println!();
    //println!("- - - - - - - - -");
    //println!("- - - - - - - - -");
    //
    //let ptrx = Movement::__get_enum_data_ptr_mut();
    //unsafe {
    //    *ptrx = 10;
    //    println!("ptrx: {:?}", *ptrx);
    //}

    // **********************************
    //let world = World::new();
    //
    //let mut entity = Entity::new_only_world(world.raw_world);
    //println!("entity: {:?}", entity.id.id);
    //entity = entity.add::<Movement>();
    //let mut entity2 = Entity::new_only_world(world.raw_world);
    //println!("entity: {:?}", entity2.id.id);
    //entity2 = entity2.add::<TrafficLight>();
    //let mut entity3 = Entity::new_only_world(world.raw_world);
    //println!("entity: {:?}", entity3.id.id);
    // **********************************

    //517,519,521
    //517,521,525

    //let mut state = EnumTest::TypeB;
    //let name = state.get_cstr_name();
    //println!("name: {:?}", name);
    //let index = state.get_enum_index();
    //println!("index: {:?}", index);
    //state = EnumTest::TypeB;
    //let name = state.get_cstr_name();
    //println!("name: {:?}", name);
    //let index = state.get_enum_index();
    //println!("index: {:?}", index);

    // The following won't work because the lifetime is incorrect:
    // let wrong: &'static str = state.as_ref();
    // using the trait implemented by the derive works however:
    //let right: &'static str = state.into();
    //let world = World::new();

    //for (index, enum_item) in Movement::iter().enumerate() {
    //    let str: &'static str = enum_item.into();
    //    println!("{:?}", enum_item);
    //    //println!("index: {}", get_index(enum_item));
    //}
    //let world2 = World::new();

    //let entity_test = EntityTest::new();
    //
    //entity_test.is_struct_component::<TypeA>();
    //entity_test.is_enum_component::<Movement>();
    //entity_test.is_not_empty_component::<TypeA>();
    //entity_test.is_empty_component::<Test2>();
    //
    ////entity_test.is_enum_component::<TypeA>(); //error: the trait bound `TypeA: CachedComponentData<EnumComponent>` is not satisfied
    ////label: the trait `CachedComponentData<EnumComponent>` is not implemented for `TypeA`
    //
    ////print id of TypeA and Test2
    //println!("TypeA id: {}", TypeA::get_id(world.raw_world));
    //println!("Test2 id: {}", Test2::get_id(world.raw_world));
    //println!("Test2 id: {}", Test2::get_id(world2.world));
    //println!("enum id: {}", Movement::get_id(world.raw_world));
    //println!("enum id: {}", Movement::get_id(world.raw_world));
    //
    //let size = std::mem::size_of::<Movement>();
    //println!("size of Movement: {}", size);

    //{
    // let mut entity = Entity::new_only_world(world.raw_world);
    // entity = entity.add::<TypeA>();
    // let test1: *const TypeA = entity.get::<TypeA>();
    // println!("test1: {:?}", test1);
    // let test2 = entity.get::<TypeB>();
    // match test2 {
    //     Some(test2) => println!("test2: {:?}", test2),
    //     None => println!("test2: does not exist"),
    // }
    // entity = entity.add::<TypeB>();
    // let test2 = entity.get::<TypeB>();
    //
    // match test2 {
    //     Some(test2) => println!("test2: {:?}", test2),
    //     None => println!("test2: does not exist"),
    // }
    //
    // entity.clear();
    // let test3 = entity.get::<TypeB>();
    // match test3 {
    //     Some(test3) => println!("test3: {:?}", test3),
    //     None => println!("test3: does not exist"),
    // }
    //
    // entity = entity.add::<Movement>();
    // let test4 = entity.get::<Movement>();
    // match test4 {
    //     Some(test4) => println!("test4: {:?}", test4),
    //     None => println!("test4: does not exist"),
    // }
    //
    // entity = entity.add::<EnumTest>();
    // let test5 = entity.get_enum::<EnumTest>();
    // match test5 {
    //     Some(test5) => println!("test5: {:?}", test5),
    //     None => println!("test5: does not exist"),
    // }
    //}
    //println!("test");

    ////////////////////////////////////
    /*
    let mut comp = Component::<Position>::new(Position { x: 10, y: 20 });

    let entity: Entity = 1;
    comp.on_add(move |position| {
        // Note the move keyword here
        println!(
            "Entity ID: {}, Position: ({}, {})",
            entity, position.x, position.y
        );
    });

    comp.trigger();

    println!("position: {:?}", comp.data);
    */

    //let mytypex: MyTypeX = MyTypeX;
    //let mytypey: MyTypeY = MyTypeY;
    //let mytypez: MyTypeZ = MyTypeZ;
    //
    //mytypez.test::<MyTypeX>();
    //
    //let ent: Entity = Entity::new(world.raw_world, 0);
    //let path = ent.get_hierachy_path_from_parent_type::<TypeB>("::", "::");
    //mytypez.test::<MyTypeY>();
}
 */

#[derive(Debug, Default, Component, Clone)]
struct Position {
    // properties for Position, for demonstration
    pub x: i32,
    pub y: i32,
}

#[allow(clippy::type_complexity)]
struct Component<T> {
    data: T,
    callback: Option<Box<dyn FnMut(&T)>>, // Note: Only one argument now
}

impl<T> Component<T> {
    fn new(data: T) -> Self {
        Component {
            data,
            callback: None,
        }
    }

    fn on_add<F: 'static + FnMut(&T)>(&mut self, func: F) {
        self.callback = Some(Box::new(func));
    }

    fn trigger(&mut self) {
        if let Some(ref mut cb) = self.callback {
            cb(&self.data);
        }
    }
}

pub trait IsEnum<T> {}

struct MyTypeX;
struct MyTypeY;

impl IsEnum<True> for MyTypeX {}
impl IsEnum<False> for MyTypeY {}

struct MyTypeZ;

impl MyTypeZ {
    pub fn test<T: IsEnum<True>>(&self) {}
    pub fn test2<T: IsEnum<False>>(&self) {}
}
fn test<T: IsEnum<True>>() {}
fn test2<T: IsEnum<False>>() {}

trait Boolean {}
struct True;
struct False;

impl Boolean for True {}
impl Boolean for False {}

trait Foo {
    type BoolType: Boolean;
}

trait Trait1 {}
trait Trait2 {}

impl<T: Foo<BoolType = True>> Trait1 for T {}
impl<T: Foo<BoolType = False>> Trait2 for T {}

/////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////

macro_rules! i32_addition {
    ($($values:expr),*) => {
        [$($values),*].iter().sum::<i32>()
    };
}

macro_rules! text_addition {
    ($value:expr) => {
        $value.to_string()
    };
    ($head:expr, $($tail:expr),*) => {
        $head.to_string() + &text_addition!($($tail),*)
    };
}

fn test_macro() {
    let r1 = i32_addition!(1, 2, 3, 5, 7);
    println!("{:?}", r1);
    let r2 = i32_addition!(1, 2, 3, 5, 7, 11, 13);
    println!("{:?}", r2);
    let r3 = text_addition!("1", "2", "3", "5", "7");
    println!("{:?}", r3);
    let r4 = text_addition!("1", "2", "3", "5", "7", "11", "13");
    println!("{:?}", r4);

    let mut pos = Position::default();
    let tuple: (&mut Position, TypeA, TypeB, TypeD) = (
        &mut pos,
        TypeA::default(),
        TypeB::default(),
        TypeD::default(),
    );

    let mut pos_ref = tuple.0;
    pos_ref.x = 10;
}

/*
**querying over 10m entities with 3 components**

mine now with the ref checks and optional
```
6.6028ms
6.4567ms
6.9281ms
6.3751ms
7.2032ms
6.4629ms
```

c++
```
8.093 ms
7.958 ms
7.606 ms
8.670 ms
7.977 ms
8.472 ms
```

(did it out of curiosity since I never did a one to one benchmark with mine)
flecs-rs without optional components or ref support/feature
```
32.637ms
32.4873ms
32.5416ms
32.5904ms
36.9944ms
32.6014ms
```

bevy-ecs
```
14.8242ms
12.6753ms
13.2552ms
13.0247ms
12.6953ms
12.9721ms
```

hecs-ecs
```
10.8313ms
10.2386ms
8.6234ms
8.8088ms
8.7602ms
9.4472ms
```
*/

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////
//
// todos / thinking
//
// * in filter struct, we can have a has_ref bool which could lead
//to some optimizations
//
// *
// wait so if a component isn't registered with world B but is with world A, is it going to override the id stored?
// need to investigate this

// Parent struct to indicate .term_at().parent() or .term_at().instanced()
// now currently you would index into a ref slice of size one with .iter

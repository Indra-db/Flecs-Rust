#![allow(dead_code)]
use crate::common_test::*;
use flecs_ecs_derive::Component;

#[repr(C)]
#[derive(Component, Debug, PartialEq)]
pub enum StandardEnum {
    Red,
    Green,
    Blue,
}

#[repr(C)]
#[derive(Component)]
pub enum AnotherEnum {
    Standing,
    Walking,
    Running,
}

#[repr(C)]
#[derive(Component)]
pub enum SparseEnum {
    Black = 1,
    White = 3,
    Grey = 5,
}

#[repr(C)]
#[derive(Component)]
pub enum EnumClass {
    Grass,
    Sand,
    Stone,
}

#[repr(C)]
#[derive(Component)]
pub enum PrefixEnum {
    PrefixEnumFoo,
    PrefixEnumBar,
}

#[repr(C)]
#[derive(Component)]
pub enum ConstantsWithNum {
    Num1,
    Num2,
    Num3,
}

#[repr(C)]
#[derive(Component)]
pub enum EnumIncorrectType {
    A,
    B,
}

#[repr(C)]
#[derive(Component)]
pub enum EnumWithLargeConstant {
    X,
    Y,
    Z = 1000,
}

#[repr(C)]
#[derive(Component)]
pub enum EnumClassWithLargeConstant {
    X,
    Y,
    Z = 1000,
}

/*
    test_int(enum_type.first_id(), Red);
    test_int(enum_type.last(), Blue);

    auto e_red = enum_type.entity(Red);
    auto e_green = enum_type.entity(Green);
    auto e_blue = enum_type.entity(Blue);

    test_assert(e_red != 0);
    test_str(e_red.path().c_str(), "::StandardEnum::Red");
    test_bool(enum_type.is_valid(Red), true);
    test_assert(e_red.get<StandardEnum>() != nullptr);
    test_assert(e_red.get<StandardEnum>()[0] == Red);

    test_assert(e_green != 0);
    test_str(e_green.path().c_str(), "::StandardEnum::Green");
    test_bool(enum_type.is_valid(Green), true);
    test_assert(e_green.get<StandardEnum>() != nullptr);
    test_assert(e_green.get<StandardEnum>()[0] == Green);

    test_assert(e_blue != 0);
    test_str(e_blue.path().c_str(), "::StandardEnum::Blue");
    test_bool(enum_type.is_valid(Blue), true);
    test_assert(e_blue.get<StandardEnum>() != nullptr);
    test_assert(e_blue.get<StandardEnum>()[0] == Blue);

    test_bool(enum_type.is_valid(Blue + 1), false);
}
*/

#[test]
fn enum_standard_enum_reflection() {
    let world = World::new();
    let entity = world.entity_from_enum(StandardEnum::Red);
    assert_eq!(
        entity.path().unwrap(),
        "::flecs::enum_test::StandardEnum::Red"
    );

    let entity2 = world.entity().set(StandardEnum::Blue);

    // TODO implement .first and .last() on all enums
    assert!(entity.is_valid());
    //let enum_comp = entity.get::<StandardEnum>().unwrap();
    entity2.set(StandardEnum::Red);
    entity2.try_get::<&StandardEnum>(|enum_comp2| {
        assert_eq!(*enum_comp2, StandardEnum::Red);
    });
    //assert!(*enum_comp == StandardEnum::Red);
    assert_eq!(entity.to_constant::<StandardEnum>(), StandardEnum::Red);

    let redd = StandardEnum::Red;

    let red = redd.id_variant(&world);
    let green = StandardEnum::Green.id_variant(&world);
    let _blue = StandardEnum::Blue.id_variant(&world);

    assert_ne!(red, 0);
    assert_ne!(green, 0);
    assert!(StandardEnum::Red.is_field_registered_as_entity());
    assert_eq!(red.path().unwrap(), "::flecs::enum_test::StandardEnum::Red");
}

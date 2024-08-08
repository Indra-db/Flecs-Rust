use core::mem::offset_of;
use flecs_ecs::prelude::*;

#[test]
fn meta_struct_field_order() {
    let world = World::new();

    #[derive(Component, Default)]
    struct Test {
        a: u32,
        b: i64,
        c: i16,
        d: i8,
        e: i64,
    }

    world
        .component::<Test>()
        .member::<u32>(("a", Count(1), offset_of!(Test, a)))
        .member::<i64>(("b", Count(1), offset_of!(Test, b)))
        .member::<i16>(("c", Count(1), offset_of!(Test, c)))
        .member::<i8>(("d", Count(1), offset_of!(Test, d)))
        .member::<i64>(("e", Count(1), offset_of!(Test, e)));

    let e = world.entity().set(Test {
        a: 10,
        b: 20,
        c: 30,
        d: 40,
        e: 50,
    });

    e.get::<&Test>(|ptr| {
        assert_eq!(ptr.a, 10);
        assert_eq!(ptr.b, 20);
        let json = world.to_expr(ptr);
        assert_eq!(json, "{a: 10, b: 20, c: 30, d: 40, e: 50}"); //if this fails, field re-ordering is not working
    });
}

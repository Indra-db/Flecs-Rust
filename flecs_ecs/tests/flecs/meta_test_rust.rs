use flecs_ecs::prelude::*;

#[test]
fn meta_struct_field_order() {
    let world = World::new();

    #[derive(Component, Default)]
    struct Test {
        a: u32,
        b: i64,
    }

    world
        .component::<Test>()
        .member::<u32>("a", 1, offset_of!(Test, a))
        .member::<i64>("b", 1, offset_of!(Test, b));

    let e = world.entity().set(Test { a: 10, b: 20 });

    e.get::<&Test>(|ptr| {
        assert_eq!(ptr.a, 10);
        assert_eq!(ptr.b, 20);
        let json = world.to_expr(ptr);
        assert_eq!(json, "{a: 10, b: 20}"); //if this fails, field re-ordering is not working
    });
}

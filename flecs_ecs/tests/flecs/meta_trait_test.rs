use flecs_ecs::prelude::meta::*;
use flecs_ecs::prelude::*;

#[derive(Component)]
#[meta]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
#[meta]
struct PositionSkipY {
    x: f32,
    #[skip]
    y: f32,
}

#[derive(Component)]
#[meta]
struct PositionSkipX {
    #[skip]
    x: f32,
    y: f32,
}

#[test]
fn test_pos() {
    let world = World::new();

    world.component::<Position>().meta();

    // Create a new entity
    let e = world.entity().set(Position { x: 2.0, y: 4.0 });

    // Convert position component to flecs expression string
    e.get::<&Position>(|p| {
        let expr: String = world.to_expr(p);
        assert_eq!(expr, "{x: 2, y: 4}");
    });
}

#[test]
fn test_pos_skip_y() {
    let world = World::new();

    world.component::<PositionSkipY>().meta();

    // Create a new entity
    let e = world.entity().set(PositionSkipY { x: 2.0, y: 4.0 });

    // Convert position component to flecs expression string
    e.get::<&PositionSkipY>(|p| {
        let expr: String = world.to_expr(p);
        assert_eq!(expr, "{x: 2}");
    });
}

#[test]
fn test_pos_skip_x() {
    let world = World::new();

    world.component::<PositionSkipX>().meta();

    // Create a new entity
    let e = world.entity().set(PositionSkipX { x: 2.0, y: 4.0 });

    // Convert position component to flecs expression string
    e.get::<&PositionSkipX>(|p| {
        let expr: String = world.to_expr(p);
        assert_eq!(expr, "{y: 4}");
    });
}

#[derive(Debug, Component)]
#[repr(C)]
#[meta]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Component)]
#[meta]
pub struct TypeWithEnum {
    pub color: Color,
}

#[test]
fn test_enum() {
    let world = World::new();

    // Register the Color component
    world.component::<Color>().meta();

    // Register the TypeWithEnum component
    world.component::<TypeWithEnum>().meta();

    assert!(world.component::<TypeWithEnum>().has::<flecs::meta::Type>());

    // Create a new entity
    let e = world
        .entity()
        .add_enum(Color::Green)
        .set(TypeWithEnum { color: Color::Blue });

    dbg!(e);

    // Convert TypeWithEnum component to flecs expression string
    e.get::<(&Color, &TypeWithEnum)>(|(color, type_enum)| {
        let expr: String = world.to_expr(color);
        assert_eq!(expr, "Green");
        let expr = world.to_expr(type_enum);
        assert_eq!(expr, "{color: Blue}");
    });
}

#[derive(Debug, Component)]
#[meta]
pub struct TypeWithString {
    pub name: String,
}

#[test]
fn test_type_w_string() {
    let world = World::new();

    // String already pre-registered

    // Register the Type containing String component
    world.component::<TypeWithString>().meta();

    assert!(
        world
            .component::<TypeWithString>()
            .has::<flecs::meta::Type>()
    );

    // Create a new entity
    let e = world.entity().set(TypeWithString {
        name: "hello".to_string(),
    });

    // Convert TypeWithEnum component to flecs expression string
    e.get::<&TypeWithString>(|str| {
        let json: String = world.to_json::<TypeWithString>(str);
        assert_eq!(json, "{\"name\":\"hello\"}");
    });
}

#[derive(Debug, Component)]
#[meta]
pub struct TypeWithVecString {
    pub names: Vec<String>,
}

#[test]
fn test_type_w_vec_string() {
    let world = World::new();

    // String already pre-registered

    // Register the Type containing Vec<String> component
    world.component::<TypeWithVecString>().meta();

    assert!(
        world
            .component::<TypeWithVecString>()
            .has::<flecs::meta::Type>()
    );

    // Create a new entity
    let e = world.entity().set(TypeWithVecString {
        names: vec!["hello".to_string(), "world".to_string()],
    });

    // Convert TypeWithVecString component to flecs json string
    e.get::<&TypeWithVecString>(|str| {
        let json: String = world.to_json::<TypeWithVecString>(str);
        assert_eq!(json, "{\"names\":[\"hello\", \"world\"]}");
    });
}

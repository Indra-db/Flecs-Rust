include!("common");

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    world.component::<First>();
    world.component::<Second>();
    world.component::<Third>();

    let query = world.query::<&Position>().group_by::<Group>().build();

    world
        .entity()
        .add::<(Group, Third)>()
        .set(Position { x: 1.0, y: 1.0 });
    world
        .entity()
        .add::<(Group, Second)>()
        .set(Position { x: 2.0, y: 2.0 });
    world
        .entity()
        .add::<(Group, First)>()
        .set(Position { x: 3.0, y: 3.0 });

    world
        .entity()
        .add::<(Group, Third)>()
        .set(Position { x: 4.0, y: 4.0 })
        .add::<Tag>();
    world
        .entity()
        .add::<(Group, Second)>()
        .set(Position { x: 5.0, y: 5.0 })
        .add::<Tag>();
    world
        .entity()
        .add::<(Group, First)>()
        .set(Position { x: 6.0, y: 6.0 })
        .add::<Tag>();

    fprintln!(&world);

    query.iter(|it, pos| {
        let group = world.entity_from_id(it.group_id());
        fprintln!(
            it,
            "Group: {:?} - Table: [{:?}]",
            group.path().unwrap(),
            it.archetype()
        );

        for i in it.iter() {
            fprintln!(it, " [{:?}]", pos[i]);
        }

        fprintln!(it);
    });

    Ok(Snap::from(&world))

    // Output:
    //  Group: "::First" - Table: [Position, (Group,First)]
    //  [Position { x: 3.0, y: 3.0 }]
    //
    //  Group: "::First" - Table: [Position, Tag, (Group,First)]
    //  [Position { x: 6.0, y: 6.0 }]
    //
    //  Group: "::Second" - Table: [Position, (Group,Second)]
    //  [Position { x: 2.0, y: 2.0 }]
    //
    //  Group: "::Second" - Table: [Position, Tag, (Group,Second)]
    //  [Position { x: 5.0, y: 5.0 }]
    //
    //  Group: "::Third" - Table: [Position, (Group,Third)]
    //  [Position { x: 1.0, y: 1.0 }]
    //
    //  Group: "::Third" - Table: [Position, Tag, (Group,Third)]
    //  [Position { x: 4.0, y: 4.0 }]
}

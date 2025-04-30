# Flecs Quickstart
This document provides a quick overview of the different features and concepts in Flecs with short examples. This is a good resource if you're just getting started or just want to get a better idea of what kind of features are available in Flecs!

## Concepts
This section contains an overview of all the different concepts in Flecs and how they wire together. The sections in the quickstart go over them in more detail and with code examples.

![Flecs Overview](img/flecs-quickstart-overview.png)

### World
The world is the container for all ECS data. It stores the entities and their components, does queries and runs systems. Typically there is only a single world, but there is no limit on the number of worlds an application can create.

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
let world = World::new();

// Do the ECS stuff
# }
```

### Entity
An entity is a unique thing in the world, and is represented by a 64 bit id. Entities can be created and deleted. If an entity is deleted it is no longer considered "alive". A world can contain up to 4 billion(!) alive entities. Entity identifiers contain a few bits that make it possible to check whether an entity is alive or not.

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
let e = world.entity();
e.is_alive(); // true!

e.destruct();
e.is_alive(); // false!
# }
```

Entities can have names which makes it easier to identify them in an application. In Rust the name can be passed using `entity_named`. If a name is provided during entity creation time and an entity with that name already exists, the existing entity will be returned.

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
let e = world.entity_named("bob");

println!("Entity name: {}", e.name());
# }
```

Entities can be looked up by name with the `lookup` function:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
# let _ = world.entity_named("bob");
let e = world.lookup("bob");
# }
```

### Id
An id is a 64 bit number that can encode anything that can be added to an entity. In flecs this can be either a component, tag or a pair. A component is data that can be added to an entity. A tag is an "empty" component. A pair is a combination of two component/tag ids which is used to encode entity relationships. All entity/component/tag identifiers are valid ids, but not all ids are valid entity identifier.

The following sections describe components, tags and pairs in more detail.

### Component
A component is a type of which instances can be added and removed to entities. Each component can be added only once to an entity (though not really, see [Pair](#pair)).

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
#[derive(Component)]
struct Position { x: f32, y: f32 }
#[derive(Default, Component)]
struct Velocity { x: f32, y: f32 }
 
# fn main() {
# let world = World::new();
let e = world.entity();

// Add a component. This creates the component in the ECS storage, but does not
// assign it with a value. To add a component, it needs to be derived with the
// Default trait otherwise it will panic at compile time.
e.add::<Velocity>();

// Set the value for the Position & Velocity components. A component will be
// added if the entity doesn't have it yet.
e.set(Position { x: 10.0, y: 20.0 })
 .set(Velocity { x: 1.0, y: 2.0 });

// Get a component
e.get::<&Position>(|p| {
    println!("Position: ({}, {})", p.x, p.y);
});

// Remove component
e.remove::<Position>();
# }
```

Each component is associated by a unique entity identifier by Flecs. This makes it possible to inspect component data, or attach your own data to components.

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Component)]
# struct Serializable;
# 
# fn main() {
# let world = World::new();
let pos_e = world.entity_from::<Position>();

println!("Name: {}", pos_e.name()); // outputs 'Name: Position'

// It's possible to add components like you would for any entity
pos_e.add::<Serializable>();
# }
```

The thing that makes an ordinary entity a component is the `Component` trait. This tells Flecs how much space is needed to store a component, and can be inspected by applications:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
let pos_e = world.entity_from::<Position>();

pos_e.get::<&flecs::Component>(|c| {
    println!("Component size: {}", c.size);
});
# }
```

Because components are stored as regular entities, they can in theory also be deleted. To prevent unexpected accidents however, by default components are registered with a tag that prevents them from being deleted. If this tag were to be removed, deleting a component would cause it to be removed from all entities. For more information on these policies, see [Relationship cleanup properties](Relationships.md#cleanup-properties).

### Tag
A tag is a component that does not have any data. In Flecs tags are empty types marked with the `Component` trait. Tags can be added & removed using the same APIs as adding & removing components, but because tags have no data, they cannot be assigned a value. Because tags (like components) are regular entities, they can be created & deleted at runtime.

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
// Option 1: create Tag as empty struct
#[derive(Component)]
struct Enemy;

// Create entity, add Enemy tag
let e = world.entity().add::<Enemy>();
e.has::<Enemy>(); // true!

e.remove::<Enemy>();
e.has::<Enemy>(); // false!

// Option 2: create Tag as entity
let enemy = world.entity();

// Create entity, add Enemy tag
let e = world.entity().add_id(enemy);
e.has_id(enemy); // true!

e.remove_id(enemy);
e.has_id(enemy); // false!
# }
```

Note that both options achieve the same effect. The only difference is that in option 1 the tag is fixed at compile time, whereas in option 2 the tag can be created dynamically at runtime.

When a tag is deleted, the same rules apply as for components (see [Relationship cleanup properties](Relationships.md#cleanup-properties)).

### Pair
A pair is a combination of two entity ids. Pairs can be used to store entity relationships, where the first id represents the relationship kind and the second id represents the relationship target (called "object"). This is best explained by an example:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
// Create Likes relationship as empty type (tag)
#[derive(Component)]
struct Likes;

// Create a small graph with two entities that like each other
let bob = world.entity();
let alice = world.entity();

bob.add_first::<Likes>(alice); // bob likes alice
alice.add_first::<Likes>(bob); // alice likes bob
bob.has_first::<Likes>(alice); // true!

bob.remove_first::<Likes>(alice);
bob.has_first::<Likes>(alice); // false!
# }
```

A pair can be encoded in a single 64 bit identifier using the `world.id_first` function:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Likes;
# 
# fn main() {
# let world = World::new();
# let bob = world.entity();
let id = world.id_first::<Likes>(bob);
# }
```

The following examples show how to get back the elements from a pair:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Likes;
# #[derive(Component)]
# struct Apples;
# 
# fn main() {
# let world = World::new();
let id = world.id_from::<(Likes, Apples)>();
if id.is_pair() {
    let relationship = id.first_id();
    let target = id.second_id();
}
# }
```

A component or tag can be added multiple times to the same entity as long as it is part of a pair, and the pair itself is unique:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
# let eats = world.entity();
# let grows = world.entity();
# let apples = world.entity();
# let pears = world.entity();
let bob = world.entity();
bob.add_id((eats, apples));
bob.add_id((eats, pears));
bob.add_id((grows, pears));

bob.has_id((eats, apples)); // true!
bob.has_id((eats, pears)); // true!
bob.has_id((grows, pears)); // true!
# }
```

The `target` function can be used to get the object for a relationship:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Likes;
# 
# fn main() {
# let world = World::new();
# let bob = world.entity();
let alice = world.entity().add_first::<Likes>(bob);
let o = alice.target::<Likes>(0); // Returns bob
# }
```

Entity relationships enable lots of interesting patterns and possibilities. Make sure to check out the [Relationships manual](Relationships.md).

### Hierarchies
Flecs has builtin support for hierarchies with the builtin `ChildOf` relationship. A hierarchy can be created with the regular relationship API or with the `child_of_id` function:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
let parent = world.entity();
let child = world.entity().child_of_id(parent);

// Deleting the parent also deletes its children
parent.destruct();
# }
```

When entities have names, they can be used together with hierarchies to generate path names or do relative lookups:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
let parent = world.entity_named("parent");
let child = world.entity_named("child").child_of_id(parent);

println!("Child path: {}", child.path().unwrap()); // output: 'parent::child'

world.lookup("parent::child"); // returns child
parent.lookup("child"); // returns child
# }
```

Queries (see below) can use hierarchies to order data breadth-first, which can come in handy when you're implementing a transform system:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
let q = world
    .query::<(&Position, &mut Position)>()
    .term_at(1)
    .parent()
    .cascade()
    // cascade queries need to be cached
    .set_cached()
    .build();

q.each(|(p, p_parent)| {
    // Do the thing
});
# }
```

### Type
The type (often referred to as "archetype") is the list of ids an entity has. Types can be used for introspection which is useful when debugging, or when for example building an entity editor. The most common thing to do with a type is to convert it to text and print it:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Default, Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Default, Component)]
# struct Velocity { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
let e = world.entity().add::<Position>().add::<Velocity>();

println!("Components: {}", e.archetype().to_string().unwrap()); // output: 'Position,Velocity'
# }
```

A type can also be iterated by an application:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Default, Component)]
# struct Position { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
# let e = world.entity().add::<Position>();
e.each_component(|id| {
    if id == world.component_id::<Position>() {
        // Found Position component!
    }
});
# }
```

### Singleton
A singleton is a single instance of a component that can be retrieved without an entity. The functions for singletons are very similar to the regular API:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Gravity { x: i32, y: i32 }
# 
# fn main() {
# let world = World::new();
// Set singleton component
world.set(Gravity { x: 10, y: 20 });

// Get singleton component
world.get::<&Gravity>(|g| {
    println!("Gravity: {}, {}", g.x, g.y);
});
# }
```

Singleton components are created by adding the component to its own entity id. The above code examples are shortcuts for these regular API calls:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Gravity { x: i32, y: i32 }
# 
# fn main() {
# let world = World::new();
let grav_e = world.entity_from::<Gravity>();

grav_e.set(Gravity { x: 10, y: 20 });

grav_e.get::<&Gravity>(|g| {
    println!("Gravity: {}, {}", g.x, g.y);
});
# }
```

The following examples show how to query for a singleton component:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Velocity { x: f32, y: f32 }
# #[derive(Component)]
# struct Gravity { x: i32, y: i32 }
# 
# fn main() {
# let world = World::new();
world
    .query::<(&Velocity, &Gravity)>()
    .term_at(1)
    .singleton()
    .build();
# }
```

### Query
Queries are the main mechanism for finding and iterating through entities. Queries are used in many parts of the API, such as for systems and observers. The following example shows a simple query:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Component)]
# struct Velocity { x: f32, y: f32 }
# #[derive(Component)]
# struct Mesh;
# #[derive(Component)]
# struct Transform;
# 
# fn main() {
# let world = World::new();
# let parent = world.entity();
// For simple queries the world::each function can be used
world.each::<(&mut Position, &Velocity)>(|(p, v)| {
    // EntityView argument is optional, use each_entity to get it
    p.x += v.x;
    p.y += v.y;
});

// More complex queries can first be created, then iterated
let q = world
    .query::<&Position>()
    .with_id((flecs::ChildOf::ID, parent))
    .build();

// Option 1: the each() callback iterates over each entity
q.each_entity(|e, p| {
    println!("{}: ({}, {})", e.name(), p.x, p.y);
}); 

// Option 2: the run() callback offers more control over the iteration
q.run(|mut it| {
    while it.next() {
        let p = it.field::<Position>(0).unwrap();

        for i in it.iter() {
            println!("{}: ({}, {})", it.entity(i).name(), p[i].x, p[i].y);
        }
    }
});
# }
```

Queries can use operators to exclude components, optionally match components or match one out of a list of components. Additionally filters may contain wildcards for terms which is especially useful when combined with pairs.

The following example shows a query that matches all entities with a parent that do not have `Position`:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
let q = world
    .query::<()>()
    .with::<(flecs::ChildOf, flecs::Wildcard)>()
    .with::<Position>()
    .set_oper(OperKind::Not)
    .build();

// Iteration code is the same
# }
```

See the [query manual](Queries.md) for more details.

### System
A system is a query combined with a callback. Systems can be either ran manually or ran as part of an ECS-managed main loop (see [Pipeline](#pipeline)). The system API looks similar to queries:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Component)]
# struct Velocity { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
// Use each_entity() function that iterates each individual entity
let move_sys = world
    .system::<(&mut Position, &Velocity)>()
    .each_iter(|it, i, (p, v)| {
        p.x += v.x * it.delta_time();
        p.y += v.y * it.delta_time();
    });

// Just like with queries, systems have both the run() and
// each() methods to iterate entities.

move_sys.run();
# }
```

Systems are stored as entities with additional components, similar to components. That means that an application can use a system as a regular entity:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Component)]
# struct Velocity { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
# let move_sys = world
#    .system::<(&mut Position, &Velocity)>()
#    .each_iter(|it, i, (p, v)| {
#        p.x += v.x * it.delta_time();
#        p.y += v.y * it.delta_time();
#    });
println!("System: {}", move_sys.name());
move_sys.add::<flecs::pipeline::OnUpdate>();
move_sys.destruct();
# }
```

### Pipeline
A pipeline is a list of tags that when matched, produces a list of systems to run. These tags are also referred to as a system "phase". Flecs comes with a default pipeline that has the following phases:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# fn main() {
# let world = World::new();
flecs::pipeline::OnLoad;
flecs::pipeline::PostLoad;
flecs::pipeline::PreUpdate;
flecs::pipeline::OnUpdate;
flecs::pipeline::OnValidate;
flecs::pipeline::PostUpdate;
flecs::pipeline::PreStore;
flecs::pipeline::OnStore;
# }
```

When a pipeline is executed, systems are ran in the order of the phases. This makes pipelines and phases the primary mechanism for defining ordering between systems. The following code shows how to assign systems to a pipeline, and how to run the pipeline with the `progress()` function:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Component)]
# struct Velocity { x: f32, y: f32 }
# #[derive(Component)]
# struct Transform { x: f32, y: f32 }
# #[derive(Component)]
# struct Mesh { id: u32 }
# 
# fn main() {
# let world = World::new();
world
    .system_named::<(&mut Position, &Velocity)>("Move")
    .kind::<flecs::pipeline::OnUpdate>()
    .each(|(p, v)| {});

world
    .system_named::<(&mut Position, &Transform)>("Transform")
    .kind::<flecs::pipeline::PostUpdate>()
    .each(|(p, t)| {});
    
world
    .system_named::<(&Transform, &mut Mesh)>("Render")
    .kind::<flecs::pipeline::OnStore>()
    .each(|(t, m)| {});

world.progress();
# }
```

Because phases are just tags that are added to systems, applications can use the regular API to add/remove systems to a phase:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Component)]
# struct Velocity { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
# let move_sys = world
#    .system::<(&mut Position, &Velocity)>()
#    .each_iter(|it, i, (p, v)| {
#        p.x += v.x * it.delta_time();
#        p.y += v.y * it.delta_time();
#    });
move_sys.add::<flecs::pipeline::OnUpdate>();
move_sys.remove::<flecs::pipeline::PostUpdate>();
# }
```

Inside a phase, systems are guaranteed to be ran in their declaration order.

### Observer
Observers are callbacks that are invoked when one or more events matches the query of an observer. Events can be either user defined or builtin. Examples of builtin events are `OnAdd`, `OnRemove` and `OnSet`.

When an observer has a query with more than one component, the observer will not be invoked until the entity for which the event is emitted satisfies the entire query.

An example of an observer with two components:

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
# #[derive(Component)]
# struct Position { x: f32, y: f32 }
# #[derive(Component)]
# struct Velocity { x: f32, y: f32 }
# 
# fn main() {
# let world = World::new();
world
    .observer_named::<flecs::OnSet, (&Position, &Velocity)>("OnSetPosition")
    .each(|(p, v)| {}); // Callback code is same as system

let e = world.entity(); // Doesn't invoke the observer
e.set(Position { x: 10.0, y: 20.0 }); // Doesn't invoke the observer
e.set(Velocity { x: 1.0, y: 2.0 }); // Invokes the observer
e.set(Position { x: 30.0, y: 40.0 }); // Invokes the observer
# }
```

### Module
A module is a function that imports and organizes components, systems, triggers, observers, prefabs into the world as reusable units of code. A well designed module has no code that directly relies on code of another module, except for components definitions. All module contents are stored as child entities inside the module scope with the `ChildOf` relationship.

```rust
# extern crate flecs_ecs;
# use flecs_ecs::prelude::*;
# 
#[derive(Component)]
struct MyModule;

impl Module for MyModule {
    fn module(world: &World) {
        world.module::<MyModule>("MyModule");
        // Define components, systems, triggers, ... as usual. They will be
        // automatically created inside the scope of the module.
    }
}

# fn main() {
# let world = World::new();
// Import code
world.import::<MyModule>();
# }

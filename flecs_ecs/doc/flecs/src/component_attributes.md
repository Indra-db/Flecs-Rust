# Component attributes: `#[flecs(...)]`

Deriving `Component` supports a rich attribute `#[flecs(...)]` to customize registration. This runs when the component is registered (e.g., when first used), and augments the component entity.

- Flecs traits (use the grouped form):
  - Single: `#[flecs(traits(Transitive, Reflexive, flecs::Exclusive))]`
  - Pair: `#[flecs(traits((With, Group)))]`
- Options
  - `#[flecs(meta)]` to enable reflection generation.
  - `#[flecs(name = "...")]` to set component name. 
  Ordering: `name` and `meta` must be the first items in any order.
- add(...)
  - Add types: `#[flecs(add(Foo, Bar))]`
  - Add pairs: `#[flecs(add((Rel, Tgt)))]`
- set(...)
  - Set single: `#[flecs(set(Value { .. }, Value2 { .. }))]`
  - Set pairs: `#[flecs(set((Rel { .. }, Tgt), (Rel, Tgt { .. })))]`
- Hooks (grouped):
  - `#[flecs(hooks(on_add(...), on_set(...), on_remove(...), on_replace(...)))]` accept a function path or inline closure.

Example:

```rust
use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
#[flecs(
  name = "Position",
  meta,
  traits(Transitive, (With, Group)),
  add((Eats, Apples)),
  set((Likes { score: 10 }, Bob)),
  hooks(
    on_add(|e, _| { println!("added Position to {}", e.name()); }),
    on_set(|e, p: &mut Position| { println!("set {:?} for {}", p, e.name()); }),
    on_remove(|e, p: &mut Position| { println!("removed {:?} from {}", p, e.name()); })
  )
)]
pub struct Position { pub x: f32, pub y: f32 }

#[derive(Component)]
pub struct Group;
#[derive(Component)]
pub struct Eats;
#[derive(Component)]
pub struct Apples; 
#[derive(Component, Debug)]
pub struct Likes { pub score: i32 }
#[derive(Component, Debug)]
pub struct Bob;

fn main() {
  let world = World::new();
  let e = world.entity_named("Bob");
  e.set(Position { x: 1.0, y: 2.0 });
  e.remove(Position::id());
}
```

Tips:
- Combine with `#[flecs(meta)]` for reflected components.
- Use `flecs::...` for clarity, or rely on auto-qualification.

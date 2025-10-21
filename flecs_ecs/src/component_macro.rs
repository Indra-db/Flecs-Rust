//! # Component Derive Macro
//!
//! ## Introduction
//! The `Component` derive macro is the primary way to register types as Flecs components in Rust. It automatically generates all the necessary boilerplate code to integrate your Rust types with Flecs ECS, including type registration, trait implementations, and component metadata.
//!
//! This manual covers all the features and attributes supported by the `Component` derive macro. For general component trait information in Flecs, see the [Flecs component trait manual](https://www.flecs.dev/flecs/md_docs_2ComponentTraits.html).
//!
//! ## Example
//! ```rust
//! # use flecs_ecs::prelude::*;
//!
//! #[derive(Component)]
//! #[flecs(
//!     name = "MyComponent",
//!     traits(Transitive, Acyclic),
//!     add(Serialize),
//!     set(ComponentData { value: 42 })
//! )]
//! struct MyComponent {
//!     value: i32,
//! }
//!
//! #[derive(Component)]
//! struct Serialize;
//! #[derive(Component, Default)]
//! struct ComponentData { value: i32 }
//! ```
//!
//! ## The Basics
//!
//! ### Simple Component
//! The most basic usage is to derive `Component` on a struct:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//! ```
//!
//! This generates all the necessary code to use `Position` as a Flecs component. The component can then be used with entities:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct Position { x: f32, y: f32 }
//! # let world = World::new();
//! let entity = world.entity()
//!     .set(Position { x: 10.0, y: 20.0 });
//! ```
//!
//! ### Tag Components
//! Components without fields are automatically treated as "tags" - zero-sized markers that don't store data:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! struct Player;
//!
//! #[derive(Component)]
//! struct Enemy;
//! ```
//!
//! Tags are more memory-efficient as they don't allocate storage:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct Player;
//! # let world = World::new();
//! let player = world.entity().add(Player::id());
//! ```
//!
//! ## The `#[flecs(...)]` Attribute
//!
//! The `#[flecs(...)]` attribute provides fine-grained control over component registration. Multiple attributes can be specified, and most take sub-attributes with various options.
//! Note that all these attributes are runtime attributes and get registered when the component gets registered internally.
//! This also means that you can add these attributes manually instead of through the derive macro.
//!
//! ### Ordering Rules
//! When using both `name` and `meta` attributes, they **must** be the first two items in the `#[flecs(...)]` attribute list (in any order). This ensures proper component initialization:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! // Valid - name and meta first
//! #[derive(Component)]
//! #[flecs(name = "CustomName", meta, traits(Sparse))]
//! struct Component1 { value: i32 }
//!
//! // Valid - meta and name first (reversed order)
//! #[derive(Component)]
//! #[flecs(meta, name = "CustomName", traits(Sparse))]
//! struct Component2 { value: i32 }
//!
//! // Invalid - traits before name/meta
//! // #[flecs(traits(Sparse), name = "CustomName")]  // Compile error!
//! ```
//!
//! ## Component Traits
//!
//! Flecs components can have traits that define their behavior and relationships. Traits are specified using the `traits(...)` attribute.
//!
//! For detailed information about what each trait does, see the [Flecs Component Traits manual](https://www.flecs.dev/flecs/md_docs_2ComponentTraits.html).
//!
//! ### Single Traits
//!
//! Single traits are specified by name within the `traits(...)` attribute:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[flecs(traits(Transitive))]
//! struct LocatedIn;
//! ```
//!
//! Available single traits include:
//! - **Relationship traits**: `Transitive`, `Reflexive`, `Symmetric`, `Acyclic`, `Traversable`
//! - **Inheritance traits**: `Inheritable`, `Final`
//! - **Special traits**: `Trait`, `Relationship`, `Target`, `Exclusive`
//! - **Storage traits**: `Sparse`, `DontFragment`
//! - **Pair traits**: `PairIsTag`, `CanToggle`
//!
//! ### Pair Traits
//!
//! Some traits require a pair of types, specified using tuple syntax within `traits(...)`:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct RequiredComponent;
//! #[derive(Component)]
//! #[flecs(traits((With, RequiredComponent)))]
//! struct MyComponent;
//! ```
//!
//! Available pair traits include:
//! - `(With, TargetType)` - Automatically adds the target component
//! - `(OneOf, GroupType)` - Mutually exclusive component group
//! - `(OnInstantiate, Override)` - Instance gets its own copy
//! - `(OnInstantiate, Inherit)` - Instance inherits from prefab
//! - `(OnInstantiate, DontInherit)` - Instance does not get the component
//!
//! ### Multiple Traits
//!
//! Multiple traits can be combined in a single `traits(...)` attribute, mixing both single and pair traits:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct RequiredTag;
//! #[derive(Component)]
//! #[flecs(traits(
//!     Acyclic,
//!     Transitive,
//!     Traversable,
//!     Inheritable,
//!     Sparse,
//!     (OnInstantiate, Inherit),
//!     (With, RequiredTag)
//! ))]
//! struct ComplexRelation;
//! ```
//!
//! This example shows a relationship component that:
//! - Cannot form cycles (`Acyclic`)
//! - Propagates through targets (`Transitive`)
//! - Can be traversed in queries (`Traversable`)
//! - Is inherited by children (`Inheritable`)
//! - Uses sparse storage (`Sparse`)
//! - Inherits from prefabs (`OnInstantiate, Inherit`)
//! - Automatically adds `RequiredTag` (`With, RequiredTag`)
//!
//! ### Using Qualified Trait Names
//!
//! Traits can be specified with full paths for clarity:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[flecs(traits(
//!     flecs::Acyclic,
//!     flecs::Transitive,
//!     flecs::Sparse,
//!     (flecs::OnInstantiate, flecs::Inherit)
//! ))]
//! struct MyRelation;
//! ```
//!
//! ## Component Name
//!
//! By default, components use their Rust type name with the fully qualified path. You can override this with the `name` attribute:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[flecs(name = "CustomComponentName")]
//! struct MyComponent {
//!     value: i32,
//! }
//! ```
//!
//! This is useful when:
//! - You want a shorter or more descriptive name in Flecs
//! - The Rust name doesn't match your ECS naming convention
//! - You need stable names across refactorings
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # #[flecs(name = "CustomComponentName")]
//! # struct MyComponent { value: i32 }
//! # let world = World::new();
//! let c = world.component::<MyComponent>();
//! assert_eq!(c.name(), "CustomComponentName");
//! ```
//!
//! ## Meta Information
//!
//! The `meta` attribute enables Flecs reflection system for your component. This allows runtime inspection of component structure:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[flecs(meta)]
//! struct Transform {
//!     position: [f32; 3],
//!     rotation: [f32; 4],
//!     scale: [f32; 3],
//! }
//! ```
//!
//! Meta information enables:
//! - Runtime component inspection
//! - Serialization/deserialization
//! - Editor integration
//! - Debug visualization
//!
//! **Requirements:**
//! - Enable the `flecs_meta` feature in your `Cargo.toml`
//! - For enums, add `#[repr(C)]` attribute
//!
//! ### Skipping Fields
//!
//! Use `#[flecs_skip]` to exclude fields from meta information:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[flecs(meta)]
//! struct Component {
//!     pub_data: i32,
//!     
//!     #[flecs_skip]
//!     private_data: String,
//! }
//! ```
//!
//! ### Meta with C-style Enums
//!
//! Enums must be C-compatible for meta information:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[repr(C)]
//! #[flecs(meta)]
//! enum GameState {
//!     Menu = 0,
//!     Playing = 1,
//!     Paused = 2,
//! }
//! ```
//!
//! ## Adding Components
//!
//! The `add(...)` attribute automatically adds other components or pairs when this component is registered:
//!
//! ### Adding Single Components
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct Tag1;
//! # #[derive(Component)]
//! # struct Tag2;
//! #[derive(Component)]
//! #[flecs(add(Tag1, Tag2))]
//! struct MyComponent;
//! ```
//!
//! ### Adding Pairs
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct Parent;
//! #[derive(Component)]
//! #[flecs(add((flecs::IsA, Parent)))]
//! struct Child;
//! ```
//!
//! When `Child` component is registered, it automatically gets `(IsA, Parent)` relationship.
//!
//! ### Mixed Add Calls
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct Tag1;
//! # #[derive(Component)]
//! # struct Tag2;
//! # #[derive(Component)]
//! # struct Parent;
//! #[derive(Component)]
//! #[flecs(add(
//!     Tag1,
//!     Tag2,
//!     (Tag1, Tag2),
//!     (flecs::ChildOf, Parent)
//! ))]
//! struct ComplexComponent;
//! ```
//!
//! ## Setting Initial Data
//!
//! The `set(...)` attribute sets initial component data during registration:
//!
//! ### Single Components
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component, Default)]
//! # struct Config { max_speed: f32 }
//! #[derive(Component)]
//! #[flecs(set(Config { max_speed: 100.0 }))]
//! struct Vehicle;
//! ```
//!
//! ### Default Construction
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component, Default)]
//! # struct Config { max_speed: f32 }
//! #[derive(Component)]
//! #[flecs(set(Config::default()))]
//! struct Vehicle;
//! ```
//!
//! ### Multiple Components
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component, Default)]
//! # struct Config { max_speed: f32 }
//! # #[derive(Component, Default)]
//! # struct Stats { health: i32 }
//! #[derive(Component)]
//! #[flecs(set(
//!     Config { max_speed: 100.0 },
//!     Stats { health: 100 }
//! ))]
//! struct Player;
//! ```
//!
//! ### Setting Pair Data
//!
//! For pairs, one element must be a value and one must be a type:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component, Default)]
//! # struct Data { value: i32 }
//! # #[derive(Component)]
//! # struct Tag;
//! # #[derive(Component)]
//! # struct Target;
//! #[derive(Component)]
//! #[flecs(set(
//!     (Data { value: 42 }, Tag),    // Data as first, Tag as second
//!     (Target, Data { value: 99 })  // Target as first, Data as second
//! ))]
//! struct MyComponent;
//! ```
//!
//! ## Component Hooks
//!
//! Hooks allow you to execute custom code during component lifecycle events. They are specified using the `hooks(...)` attribute.
//!
//! ### Available Hooks
//!
//! #### `on_add`
//! Called when the component is added to an entity:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! fn init_component(entity: EntityView<'_>, component: &mut MyComponent) {
//!     component.initialized = true;
//! }
//!
//! #[derive(Component, Default)]
//! #[flecs(hooks(on_add(init_component)))]
//! struct MyComponent {
//!     initialized: bool,
//! }
//! ```
//!
//! #### `on_set`
//! Called when the component data is modified:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! fn validate_position(entity: EntityView<'_>, pos: &mut Position) {
//!     pos.x = pos.x.clamp(-100.0, 100.0);
//!     pos.y = pos.y.clamp(-100.0, 100.0);
//! }
//!
//! #[derive(Component, Default)]
//! #[flecs(hooks(on_set(validate_position)))]
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//! ```
//!
//! #### `on_remove`
//! Called when the component is removed from an entity:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! fn cleanup_resource(entity: EntityView<'_>, res: &mut ResourceHandle) {
//!     // Clean up external resources
//! }
//!
//! #[derive(Component, Default)]
//! #[flecs(hooks(on_remove(cleanup_resource)))]
//! # struct ResourceHandle;
//! ```
//!
//! #### `on_replace`
//! Called when component data is replaced (receives both old and new values):
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! fn transfer_state(
//!     entity: EntityView<'_>,
//!     old: &mut State,
//!     new: &mut State
//! ) {
//!     new.previous_value = old.current_value;
//! }
//!
//! #[derive(Component, Default)]
//! #[flecs(hooks(on_replace(transfer_state)))]
//! # struct State { previous_value: i32, current_value: i32 }
//! ```
//!
//! ### Inline Hook Closures
//!
//! Hooks can also be specified as inline closures:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component, Default)]
//! # struct Counter { count: u32 }
//! #[derive(Component, Default)]
//! #[flecs(hooks(on_add(|entity, component: &mut Tracked| {
//!     // Initialization code
//! })))]
//! struct Tracked {
//!     id: u32,
//! }
//! ```
//!
//! ### Multiple Hooks
//!
//! You can specify multiple hooks in a single `hooks(...)` attribute:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component, Default)]
//! #[flecs(hooks(
//!     on_add(|e, c: &mut Managed| { /* init */ }),
//!     on_remove(|e, c: &mut Managed| { /* cleanup */ })
//! ))]
//! struct Managed {
//!     resource: Option<u32>,
//! }
//! ```
//!
//! ## Component Registration Callbacks
//!
//! The `on_registration` attribute allows you to execute custom code when a component is registered with the world.
//! This is useful for performing one-time initialization, adding additional traits, or configuring the component entity.
//!
//! ### Basic Usage
//!
//! When using the `on_registration` attribute, your component must implement the `OnComponentRegistration` trait:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! #[flecs(on_registration)]
//! struct MyComponent {
//!     value: i32,
//! }
//!
//! impl OnComponentRegistration for MyComponent {
//!     fn on_component_registration(world: WorldRef, component_id: Entity) {
//!         // Custom registration logic here
//!         let component = world.component_untyped_from(component_id);
//!         // ...
//!     }
//! }
//! ```
//!
//! ### Registering Singleton Data
//!
//! Initialize singleton components during registration:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component, Default)]
//! # struct Counter { count: u32 }
//! #[derive(Component, Default)]
//! #[flecs(on_registration, traits(Singleton))]
//! struct GameConfig {
//!     max_players: u32,
//! }
//!
//! impl OnComponentRegistration for GameConfig {
//!     fn on_component_registration(world: WorldRef, _component_id: Entity) {
//!         // Initialize global configuration
//!         world.set(GameConfig { max_players: 4 });
//!     }
//! }
//! ```
//!
//! ### Important Notes
//!
//! - The callback is executed **once** when the component is first registered
//! - Registration happens lazily when the component is first used
//!
//! ## Combining Attributes
//!
//! All `#[flecs(...)]` attributes can be combined. Here's a complex example:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! # #[derive(Component)]
//! # struct RequiredTag;
//! # #[derive(Component, Default)]
//! # struct InitData { value: i32 }
//! #[derive(Component)]
//! #[flecs(
//!     name = "GameEntity",
//!     meta,
//!     on_registration,
//!     traits(
//!         Sparse,
//!         (OnInstantiate, Inherit),
//!         (With, RequiredTag)
//!     ),
//!     set(InitData { value: 42 }),
//!     hooks(
//!         on_add(|e, c: &mut ComplexComponent| {
//!             // Initialization code here
//!         }),
//!         on_remove(|e, c: &mut ComplexComponent| {
//!             // Cleanup code here
//!         })
//!     )
//! )]
//! struct ComplexComponent {
//!     data: i32,
//! }
//!
//! impl OnComponentRegistration for ComplexComponent {
//!     fn on_component_registration(world: WorldRef, component_id: Entity) {
//!         // Custom registration logic
//!         println!("ComplexComponent registered with id: {:?}", component_id);
//!     }
//! }
//! ```
//!
//! **Important:** Remember that `name` and `meta` must appear first if present!
//!
//! ## Generic Components
//!
//! Generic types can derive `Component`, but with some limitations:
//!
//! ```rust
//! # use flecs_ecs::prelude::*;
//! #[derive(Component)]
//! struct Container<T: 'static + core::marker::Sync + core::marker::Send> {
//!     value: T,
//! }
//! ```
//!
//! Each concrete instantiation (e.g., `Container<i32>`, `Container<f32>`) becomes a separate component type in Flecs and are stored less efficiently than non-generic components.
//!
//! ## Error Handling
//!
//! The macro generates compile-time errors for invalid configurations:
//!
//! ```compile_fail
//! # use flecs_ecs::prelude::*;
//! // Error: name and meta must be first
//! #[derive(Component)]
//! #[flecs(traits(Sparse), name = "Invalid")]
//! struct BadOrdering;
//! ```
//!
//! ```compile_fail
//! # use flecs_ecs::prelude::*;
//! // Error: Meta on enum requires #[repr(C)]
//! #[derive(Component)]
//! #[flecs(meta)]
//! enum BadEnum {
//!     Variant1,
//!     Variant2,
//! }
//! ```

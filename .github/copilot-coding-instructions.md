# General Coding Instructions for Flecs Rust

## Project Context
This is the Rust API for Flecs ECS (Entity Component System). The project provides safe Rust bindings around the C Flecs library with a focus on ergonomic builder APIs while maintaining C-level performance.

## Core Principles

### Safety First
- Wrap all unsafe operations in safe APIs
- Use proper lifetime annotations for entity/component references
- Document safety invariants with `// SAFETY:` comments

### Builder API Patterns
Follow the established fluent builder pattern:
```rust
world.entity_named("Player")
    .set(Position { x: 0.0, y: 0.0 })
    .add(Active)
    .child_of(scene);
```

### Error Handling
- Use `Result<T, E>` for fallible operations
- Provide `try_` variants for optional operations
- Panic only on impossible states with descriptive messages
- Use descriptive error types (not just strings)

## Code Style

### Rust Conventions
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Prefer `impl Trait` over trait objects when possible
- Use `#[must_use]` for builders and important return values

### Flecs-Specific Patterns
- Components use `#[derive(Component)]`
- Use `ComponentId` trait for type safety
- Prefer `world.component::<T>()` over raw IDs
- Use feature flags for optional functionality

### Documentation
- Document all public APIs with examples
- Use `# Safety` sections for unsafe functions
- Provide usage examples for complex APIs

## Feature Flag Guidelines
- Use `#[cfg(feature = "...")]` for conditional compilation
- Document feature requirements in API docs

## Testing Patterns
- Use `rstest` for parameterized tests
- Test with different feature flag combinations
- Include integration tests for complex workflows
- Use snapshot testing with `insta` for output validation

## Performance Considerations
- Minimize C API calls by batching operations
- Use cached queries for frequently accessed data
- Prefer stack allocation over heap when possible
- Profile with criterion benchmarks for performance-critical code

## Common Patterns

### Component Access
```rust
// Preferred: callback pattern
entity.get::<&Position>(|pos| {
    println!("Position: {:?}", pos);
});

// For optional components
entity.try_get::<Option<&Velocity>>(|vel| {
    if let Some(vel) = vel {
        // handle velocity
    }
});
```

### Query Building
```rust
// Use builder pattern
let query = world.query::<(&Position, &mut Velocity)>()
    .with(Active)
    .without(Disabled)
    .build();

// Or DSL macro
let query = query!(world, &Position, &mut Velocity, Active, !Disabled).build();
```

### System Registration
```rust
world.system::<(&mut Position, &Velocity)>()
    .each(|(pos, vel)| {
        pos.x += vel.x;
        pos.y += vel.y;
    });
```

## Integration Points
- Always validate C API return values
- Use `NonNull<T>` for C pointers that shouldn't be null
- Convert C strings using proper encoding
- Handle C errors gracefully with Rust error types

When in doubt, prioritize safety and clarity over performance optimizations. Document any non-obvious decisions thoroughly.
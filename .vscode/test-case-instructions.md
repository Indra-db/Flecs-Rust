# Test Case Instructions for Flecs Rust

## Testing Philosophy

Write comprehensive tests that validate both functionality and safety of the Rust API while ensuring compatibility with the underlying C Flecs library.

## Test Organization

### Test Location Structure
```
flecs_ecs/tests/flecs/           # Integration tests
├── main.rs                      # Test module organization
├── common_test.rs               # Shared test components and utilities
├── entity_test.rs               # Core entity functionality
├── entity_rust_test.rs          # Rust-specific entity patterns
├── query_test.rs                # Query functionality
├── system_test.rs               # System functionality
├── component_test.rs            # Component registration and lifecycle
├── safety/                      # Safety-related tests (conditional)
└── ...
```

### Common Test Setup
- Use `common_test.rs` for shared components and utilities
- Always include `#![allow(dead_code)]` at the top of test files
- Use `crate::common_test::*` import for shared test components
- Initialize crash handler with `#[ctor::ctor]` in `common_test.rs`

## Test Patterns

### Basic Test Structure
```rust
#[test]
fn descriptive_test_name() {
    let world = World::new();
    
    // Setup
    let entity = world.entity_named("TestEntity");
    entity.set(Position { x: 0, y: 0 });
    
    // Action
    entity.set(Velocity { x: 1, y: 2 });
    
    // Verification
    assert!(entity.has(Velocity::id()));
    entity.get::<&Velocity>(|vel| {
        assert_eq!(vel.x, 1);
        assert_eq!(vel.y, 2);
    });
}
```

### Parameterized Tests with rstest
Use `rstest` for testing across different component types:

```rust
use rstest::rstest;
use rstest_reuse::*;

#[derive(Clone, Copy, Debug)]
enum ComponentType {
    Fragment,
    Sparse,
    DontFragment,
}

#[template]
#[rstest(
    case::fragment(ComponentType::Fragment),
    case::sparse(ComponentType::Sparse),
    case::dont_fragment(ComponentType::DontFragment)
)]
fn component_types(#[case] ty: ComponentType) {}

#[apply(component_types)]
fn test_component_behavior(ty: ComponentType) {
    let world = World::new();
    set_component_type::<Position>(&world, ty);
    // Test logic here
}
```

### Safety Tests
For safety-critical functionality, use feature-gated tests:

```rust
#[cfg(feature = "flecs_safety_locks")]
#[test]
#[should_panic]
fn test_component_aliasing_prevention() {
    // Test that multiple mutable references to same component are prevented
}
```

### Panic Tests
Test error conditions with descriptive panic messages:

```rust
#[test]
#[should_panic(expected = "Component not found")]
fn test_missing_component_access() {
    let world = World::new();
    let entity = world.entity();
    let _ = entity.cloned::<&Position>(); // Should panic
}
```

## Test Categories

### 1. Entity Tests
- Entity creation (named, anonymous, nested)
- Component addition/removal
- Entity relationships and hierarchies
- Entity lifecycle (creation, deletion, reuse)

### 2. Component Tests
- Component registration with derive macro
- Component traits (`#[flecs(...)]` attributes)
- Component lifecycle hooks
- Enum component handling
- Generic component support

### 3. Query Tests
- Query building and execution
- Query caching behavior
- Complex query patterns (relationships, variables)
- Query performance characteristics

### 4. System Tests
- System registration and execution
- System ordering and dependencies
- System with different component patterns
- Observer pattern tests

### 5. Safety Tests
- Memory safety validation
- Component aliasing prevention
- FFI boundary safety
- Lifetime management

### 6. Feature Flag Tests
Test behavior across different feature combinations:

```rust
#[test]
#[cfg(feature = "flecs_meta")]
fn test_meta_functionality() {
    // Test meta/reflection features
}

#[test]
#[cfg(not(feature = "flecs_manual_registration"))]
fn test_automatic_registration() {
    // Test automatic component registration
}
```

## Test Data and Fixtures

### Common Test Components
Use components from `common_test.rs`:

```rust
// Basic components
Position { x: f32, y: f32 }
Velocity { x: f32, y: f32 }
Count(i32)
Value { value: i32 }

// Relationship components
Likes, Eats, Apples, Pears

// Reference components
SelfRef { value: Entity }
EntityRef { value: Entity }
```

### Test Utilities
- Use `World::new()` for isolated test environments
- Prefer descriptive entity names for debugging
- Use assertion helpers for common patterns

## Performance and Benchmark Tests

### Benchmark Structure
```rust
// In benches/ directory
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn entity_creation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flecs");
    group.bench_function("entity_creation", |b| {
        let world = World::new();
        b.iter(|| {
            black_box(world.entity());
        });
    });
}
```

### Integration with Examples
Tests can reference examples for validation:

```rust
#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test_hello_world_example() {
    // Test that examples work correctly
}
```

## Best Practices

### Test Naming
- Use descriptive names: `entity_new_with_nested_scope_hierarchy`
- Group related tests in modules
- Use snake_case consistently

### Test Isolation
- Each test should create its own `World`
- Avoid global state between tests
- Use feature flags appropriately

### Error Testing
- Test both success and failure paths
- Use appropriate panic messages
- Test edge cases and boundary conditions

### Documentation
- Include doc comments for complex test scenarios
- Reference corresponding C++ tests when applicable
- Document any platform-specific behavior

### Assertions
- Use specific assertions (`assert_eq!` vs `assert!`)
- Include helpful failure messages
- Test multiple aspects of functionality

## Feature-Specific Testing

### Meta/Reflection Tests
```rust
#[cfg(feature = "flecs_meta")]
#[test]
fn test_component_metadata() {
    let world = World::new();
    let comp = world.component::<Position>();
    // Test metadata operations
}
```

### Safety Lock Tests
```rust
#[cfg(feature = "flecs_safety_locks")]
mod safety_tests {
    // Component aliasing prevention tests
    // Table locking mechanism tests
}
```

When writing tests, prioritize safety validation, API usability, and compatibility with Flecs C behavior patterns.
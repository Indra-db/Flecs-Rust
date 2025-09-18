# Code Review Instructions

## Focus Areas for Flecs Rust API

### Safety and Memory Management
- **FFI Safety**: Check for proper unsafe blocks with safety comments
- **Lifetime Management**: Verify entity/component lifetimes are correctly managed
- **Component Aliasing**: Look for potential multiple mutable references to same component
- **Null Pointer Handling**: Ensure C pointers are properly validated before use

### API Design Patterns
- **Builder Pattern Consistency**: Verify fluent APIs follow established patterns
- **Error Handling**: Check for proper Result types and panic conditions

### Performance Considerations
- **C API Efficiency**: Check if C API calls are minimized and batched when possible

### Flecs-Specific Patterns

### Testing and Documentation
- **Test Coverage**: Look for corresponding tests, especially for unsafe operations
- **Example Code**: Check if complex APIs have usage examples
- **Safety Documentation**: Ensure unsafe operations are well-documented

### Code Quality
- **Rust Idioms**: Prefer Rust patterns over direct C translations
- **Type Safety**: Use strong typing over raw pointers where possible
- **Error Messages**: Provide helpful error messages for common mistakes
- **Performance**: Flag potentially expensive operations in hot paths

## Review Template

When reviewing code, structure feedback as:

1. **Safety Issues** (Critical)
2. **API Design** (Important) 
3. **Performance** (Consider)
4. **Style/Documentation** (Minor)

Focus on Flecs ECS domain knowledge and Rust safety patterns.
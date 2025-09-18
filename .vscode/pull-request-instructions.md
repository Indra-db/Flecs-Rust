# Pull Request Description Instructions

## PR Title Format
Use conventional commit format: `<type>(<scope>): <description>`

## Required Sections

### Summary
- **What**: Brief description of changes
- **Why**: Motivation and context for the change
- **Impact**: What users/developers should know

### Changes Made
- List of key modifications
- New APIs or features added
- Breaking changes (if any)
- Files/modules affected

### Testing
- How the changes were tested
- New test cases added
- Manual testing performed
- Feature flag combinations tested

### Flecs-Specific Considerations

#### Safety Changes
- Document any unsafe block additions/modifications
- Explain memory safety guarantees
- Note any aliasing prevention measures

#### API Changes
- Document new builder methods or patterns
- Explain backward compatibility
- Note any deprecations

#### Performance Impact
- Benchmark results if applicable
- Query optimization changes
- C API call optimizations

#### Feature Flags
- List affected feature flags
- Compatibility with different combinations
- New conditional compilation added

### Breaking Changes
If this PR contains breaking changes:
- **BREAKING CHANGE:** prefix in title
- Clear migration guide
- Justification for the break
- Alternative approaches considered

### Dependencies
- New dependencies added
- Version updates
- Feature flag dependencies

### Documentation
- README updates needed
- Doc comments added/updated
- Examples added/modified
- Book chapters affected

### Checklist Template
```markdown
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No unsafe code without safety comments
- [ ] Feature flags tested
- [ ] Performance impact considered
- [ ] Breaking changes documented
- [ ] Examples updated if needed
```

## Example Structure

```markdown
feat(query): add cached query builder pattern

## Summary
Adds a new builder pattern for cached queries to improve performance for frequently reused queries.

## Changes Made
- New `CachedQueryBuilder` in `core/query_builder.rs`
- Helper methods for common query patterns
- Integration with existing `QueryBuilder`
- Performance benchmarks

## Testing
- Unit tests for builder pattern
- Integration tests with various feature flags
- Benchmark comparison with uncached queries
- Manual testing with example applications

## Performance Impact
- ~15% improvement for repeated query execution
- Minimal memory overhead for caching
- See `benches/cached_query_bench.rs` for details

## Breaking Changes
None - this is additive API only.
```
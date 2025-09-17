# Commit Message Instructions

Use Conventional Commits (Angular style). Output only the commit message.

## Format
```
<type>(<scope>): <short imperative summary>

<body>

<footer>
```

For breaking changes:
```
❗<type>(<scope>)!: <short imperative summary>

<body>

BREAKING CHANGE: <concise explanation>
```

## Rules
- type ∈ {feat, fix, docs, refactor, perf, test, build, ci, chore, revert}
- scope = affected module/path (optional but preferred). Use kebab-case. No spaces.
- summary ≤ 50 chars, imperative mood, lowercase, no trailing period.
- Wrap body at ~72 chars. Explain what/why vs. how. Note impacts/alternatives.
- Use `!` after scope for breaking changes ONLY and include a footer line:
  BREAKING CHANGE: <concise explanation>
- For breaking changes, start the commit message with ❗ (red exclamation mark emoji)
- DO NOT use `!` for non-breaking changes (regular features, fixes, docs, etc.)
- Reference issues in footer: Closes #123, Relates-to #456
- One logical change per commit. Do not include WIP noise.
- No marketing language. Be precise and factual.

## Examples
```
feat(auth): add TOTP-based MFA
fix(payments): retry webhook on 5xx with backoff
perf(image-cache): avoid recomputing keys
❗refactor(api)!: remove deprecated v1 endpoints
docs(readme): add Apple Silicon setup
```

## Template to follow exactly

For regular changes (most common):
```
<type>(<scope>): <summary>

<why/what changed; user impact; risks>

<issue refs if any>
```

For breaking changes only:
```
❗<type>(<scope>)!: <summary>

<why/what changed; user impact; risks>

BREAKING CHANGE: <concise explanation>
<issue refs if any>
```
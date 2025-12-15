# Design: Prompt Context Injection

## Context

The `UserPromptSubmit` hook fires when users submit prompts to Claude Code. Currently, this hook only performs validation and logging. This design document outlines how to extend it to support conditional context injection based on pattern matching.

## Goals

- Enable pattern-based matching on user prompt content
- Support prepending context/instructions when patterns match
- Maintain backward compatibility with existing configurations
- Keep the feature simple and performant

## Non-Goals

- Blocking or rejecting prompts (different feature)
- Modifying the original prompt content (only prepending)
- Complex template engines or variable substitution beyond `@file` references

## Decisions

### Decision 1: Configuration Structure

**Choice**: Use a dedicated `userPromptSubmit` section with `contextRules` array

**Rationale**:
- Follows the pattern established by `preToolUse.toolUsageValidation`
- Keeps prompt-related configuration in a clearly named section
- Allows future expansion (e.g., `blockRules`, `transformRules`)

**Alternatives considered**:
- Nesting under existing `preToolUse` section - rejected as it conflates tool validation with prompt processing
- Top-level `contextInjection` section - rejected as it's less discoverable

### Decision 2: Pattern Matching Engine

**Choice**: Use Rust's `regex` crate for pattern matching

**Rationale**:
- Already a dependency in the project
- Powerful pattern matching including case-insensitive mode
- Well-understood regex syntax

**Alternatives considered**:
- Glob patterns (like `preToolUse.uneditableFiles`) - rejected as too limited for text matching
- Custom DSL - rejected as over-engineering

### Decision 3: Context Injection Method

**Choice**: Return augmented prompt via `HookResult` with new `system_prompt` field

**Rationale**:
- The hook result already has a `message` field that Claude sees
- Adding a `system_prompt` field allows prepending context naturally
- Claude Code already supports system reminders via `<system-reminder>` tags

**Implementation**:
```rust
pub struct HookResult {
    pub message: Option<String>,
    pub blocked: Option<bool>,
    pub system_prompt: Option<String>, // NEW: Context to prepend
}
```

### Decision 4: Multiple Pattern Matches

**Choice**: Concatenate all matching contexts in order

**Rationale**:
- Users may want multiple contexts to apply (e.g., "auth sidebar" matches both auth and sidebar patterns)
- Order is deterministic based on config order
- Simple to understand and debug

**Alternatives considered**:
- First-match-wins - rejected as users often want layered contexts
- Allow explicit ordering with priority field - rejected as over-engineering for v1

### Decision 5: File Reference Syntax

**Choice**: Use `@path/to/file` syntax, expand at runtime

**Rationale**:
- Familiar syntax from Claude Code's existing context file references
- Allows dynamic content from files
- Files are resolved relative to config file location

**Alternatives considered**:
- Inline-only content - rejected as promotes duplication
- Include directive like `!include file.md` - rejected as non-standard

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Regex performance on long prompts | Low | Patterns are typically simple; regex is compiled once |
| Missing file references | Medium | Validate at config load time, warn if file doesn't exist |
| Context bloat | Medium | Document best practices; suggest keeping contexts concise |
| Breaking existing behavior | Low | New section, no existing configs affected |

## Open Questions

1. Should we support glob patterns as alternative to regex? (Recommend: defer to v2 if requested)
2. Should context injection be logged/notified separately? (Recommend: yes, for debugging)
3. Should we add a `enabled` field per rule? (Recommend: yes, for easy toggling)

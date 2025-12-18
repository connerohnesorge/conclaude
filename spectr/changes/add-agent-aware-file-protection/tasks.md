# Tasks: Add Agent-Aware File Protection

## 1. Configuration Schema

### 1.1 Extend UnEditableFileRule Struct
- [ ] 1.1.1 Add `agent: Option<String>` field to `UnEditableFileRule::Detailed` in `src/config.rs`
- [ ] 1.1.2 Add `agent()` accessor method returning `Option<&str>`
- [ ] 1.1.3 Update `#[serde]` attributes for proper YAML serialization (camelCase)

### 1.2 Update JSON Schema
- [ ] 1.2.1 Add `agent` property to uneditableFiles detailed format in `schemas/conclaude.schema.json`
- [ ] 1.2.2 Add description explaining glob pattern support for agent matching
- [ ] 1.2.3 Add examples showing `*`, exact match, and glob patterns

## 2. Agent Detection

### 2.1 Agent Context Resolution
- [ ] 2.1.1 Create `detect_current_agent()` function in `src/hooks.rs`
- [ ] 2.1.2 For main session (no subagent context), return `"main"`
- [ ] 2.1.3 For subagent context, extract agent type from transcript using existing `extract_agent_name_from_transcript()`
- [ ] 2.1.4 Handle edge cases: missing transcript, parse errors (default to "main" or "*")

### 2.2 Agent Pattern Matching
- [ ] 2.2.1 Create `matches_agent_pattern(agent_name: &str, pattern: &str) -> bool` function
- [ ] 2.2.2 Handle `*` wildcard (matches all agents)
- [ ] 2.2.3 Use `glob::Pattern` for pattern matching (consistent with file patterns)
- [ ] 2.2.4 Handle invalid patterns gracefully (log warning, default to no match)

## 3. Hook Integration

### 3.1 Modify uneditableFiles Checking
- [ ] 3.1.1 Update `check_file_validation_rules()` to detect current agent context once per invocation
- [ ] 3.1.2 For each uneditableFiles rule, check agent match before file pattern match
- [ ] 3.1.3 Skip rule if agent doesn't match (rule not applicable to current agent)
- [ ] 3.1.4 Default to `*` when rule has no agent field (backward compatible)

### 3.2 Error Messages
- [ ] 3.2.1 Include agent context in blocked operation messages when agent-specific rule triggered
- [ ] 3.2.2 Distinguish between "all agents" vs "specific agent" blocking in error messages

## 4. Testing

### 4.1 Unit Tests
- [ ] 4.1.1 Test `matches_agent_pattern()` with various patterns (`*`, exact, glob)
- [ ] 4.1.2 Test `detect_current_agent()` for main session and subagent contexts
- [ ] 4.1.3 Test UnEditableFileRule parsing with and without agent field
- [ ] 4.1.4 Test backward compatibility: rules without agent field apply to all agents

### 4.2 Integration Tests
- [ ] 4.2.1 Test preToolUse hook with agent-specific rules
- [ ] 4.2.2 Test that main session blocked by agent="main" rules
- [ ] 4.2.3 Test that coder subagent blocked by agent="coder" rules
- [ ] 4.2.4 Test glob patterns like agent="code*" matching "coder", "coder-v2"
- [ ] 4.2.5 Test that agent="coder" does NOT block main session or tester

## 5. Documentation

### 5.1 Configuration Examples
- [ ] 5.1.1 Update `src/default-config.yaml` with agent-aware uneditableFiles examples
- [ ] 5.1.2 Add comments explaining agent field behavior
- [ ] 5.1.3 Document the "main" agent identifier for orchestrator session

### 5.2 Spec Updates
- [ ] 5.2.1 Update preToolUse spec with agent-aware requirements
- [ ] 5.2.2 Add scenarios for agent matching behavior

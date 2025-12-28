## 1. CLI Changes
- [x] 1.1 Add `--agent` optional argument to HookCommand enum in src/main.rs
- [x] 1.2 Parse `--agent` flag in hook command handling
- [x] 1.3 Pass agent name to hook handlers

## 2. Hook Handler Updates
- [x] 2.1 Update handle_pre_tool_use() to accept optional agent parameter
- [x] 2.2 Update handle_post_tool_use() to accept optional agent parameter
- [x] 2.3 Update handle_stop() to accept optional agent parameter
- [x] 2.4 Update all other hook handlers similarly
- [x] 2.5 Export CONCLAUDE_AGENT_NAME when agent flag provided
- [x] 2.6 Skip transcript parsing when agent name provided via flag

## 3. Initialization Updates
- [x] 3.1 Add agent file discovery to init (glob .claude/agents/**/*.md)
- [x] 3.2 Parse YAML frontmatter from agent files
- [x] 3.3 Extract agent name from frontmatter (or derive from filename)
- [x] 3.4 Generate hooks section with all hook types
- [x] 3.5 Write updated frontmatter back to agent files
- [x] 3.6 Remove SubagentStart/SubagentStop from settings.json template

## 4. Code Cleanup
- [x] 4.1 Remove save_agent_context() function
- [x] 4.2 Remove load_agent_context() function
- [x] 4.3 Remove extract_agent_name_from_transcript() function
- [x] 4.4 Remove session file temp directory usage
- [x] 4.5 Simplify SubagentStop handler (no transcript parsing)

## 5. Testing
- [x] 5.1 Add tests for --agent flag parsing
- [x] 5.2 Add tests for agent frontmatter injection
- [x] 5.3 Update existing hook tests to use --agent flag
- [x] 5.4 Test init with various agent file formats

## 6. Documentation
- [x] 6.1 Update configuration schema docs
- [x] 6.2 Add migration guide for existing users

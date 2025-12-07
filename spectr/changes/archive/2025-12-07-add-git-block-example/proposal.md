# Add Git Command Blocking Example to Default Configuration

## Why

The default configuration file (`src/default-config.yaml`) provides comprehensive examples for most features but is missing a practical example of blocking git shell commands using `toolUsageValidation`. Users need guidance on how to prevent Claude Code from executing potentially dangerous git operations (like `git push --force`) or restrict git usage entirely. Adding this example addresses issue #111 and improves discoverability of this security feature.

## What Changes

- Add commented example to `toolUsageValidation` section in `src/default-config.yaml` showing how to block git commands
- Include both specific command blocking (e.g., `git push --force*`) and wildcard git blocking examples
- Follow existing documentation patterns with clear explanatory comments
- Position the example logically within the existing `toolUsageValidation` examples section

**Example to add:**
```yaml
# toolUsageValidation:
#   # Block dangerous git force push operations
#   - tool: "Bash"
#     commandPattern: "git push --force*"
#     action: "block"
#     message: "Force push is not allowed - please use regular push"
#
#   # Block all git commands (uncomment to completely disable git via Bash tool)
#   # - tool: "Bash"
#   #   commandPattern: "git *"
#   #   action: "block"
#   #   message: "Git commands are not permitted in this session"
```

## Impact

- **Affected specs**: configuration-defaults (documentation update)
- **Affected code**: `src/default-config.yaml` (add commented examples)
- **User experience**: Users can easily discover and copy examples for blocking git commands
- **Documentation**: Improves completeness of default config documentation
- **Security**: Helps users understand how to restrict git operations for safety

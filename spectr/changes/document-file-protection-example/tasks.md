# Tasks

## Implementation Tasks

1. **Add file protection scenario to README.md**
   - Insert new "Scenario: The Test Integrity Enforcer" after Scenario 2
   - Include problem statement about visual regression test tolerance
   - Show YAML configuration with `uneditableFiles: ["visual-regression/**"]`
   - Display actual error output from conclaude blocking the edit
   - Explain the outcome: test integrity preserved, Claude must fix CSS
   - Validate: Ensure scenario format matches existing scenarios 1 and 2

2. **Enhance PreToolUse documentation in hooks guide**
   - Add "Real-World Example: Protecting Test Integrity" subsection to PreToolUse section
   - Include context about visual regression testing and why tolerance is problematic
   - Show the error message in a code block with proper formatting
   - Explain the pattern matching: how `visual-regression/**` blocks nested files
   - Connect mechanism (pattern blocking) to purpose (value enforcement)
   - Validate: Ensure markdown renders correctly and flows with existing content

3. **Review and validate documentation changes**
   - Read through README.md with new scenario in context
   - Read through hooks.md guide with enhanced PreToolUse section
   - Check that terminology is consistent across both additions
   - Verify code blocks use correct fencing (\`\`\`yaml, \`\`\`bash)
   - Confirm no contradictions with existing documentation
   - Test that examples are clear and educational

## Validation Tasks

4. **Verify error message accuracy**
   - Confirm error output format matches actual conclaude PreToolUse output
   - Check that field names (tool_name, file_path, pattern) are correct
   - Validate that pattern `visual-regression/**` is valid glob syntax
   - Ensure error message text matches actual hook implementation

5. **Ensure consistency and quality**
   - Verify scenario follows README.md formatting conventions
   - Check that hooks guide addition follows docs site style
   - Confirm examples are educational and actionable
   - Validate that examples help users understand *when* to use file protection

## Dependencies
- None (documentation-only change)

## Notes
- All changes are additions to existing documentation
- No code changes required
- No breaking changes to configuration or behavior
- Examples should be copy-pasteable where applicable

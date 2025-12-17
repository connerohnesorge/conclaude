# Proposal: Document File Protection Example

## Summary
Add a concrete, real-world example demonstrating how `uneditableFiles` pattern blocking enforces adherence to project values. The example shows Claude attempting to modify visual regression test code and being blocked by conclaude's file protection.

## Motivation
The README.md and documentation currently explain *what* file protection does, but lack a compelling real-world example showing *why* it matters. The provided bash output shows a perfect use case: Claude trying to add "tolerance" to visual regression tests (which defeats their purpose) and conclaude blocking it.

This example demonstrates:
1. **Value enforcement**: Visual regression tests require exact matches—adding tolerance undermines testing integrity
2. **Protection in action**: The `uneditableFiles` pattern `visual-regression/**` blocks the edit attempt
3. **Clear feedback**: The error message shows exactly what was blocked and why

## Goals
- Add the file protection example to README.md in the "Real-World Scenarios" section
- Add the example to docs site under the hooks guide
- Show concrete error output demonstrating the protection mechanism
- Highlight how file protection enforces project values beyond simple "don't edit this"

## Non-Goals
- Changing any implementation code
- Modifying existing configuration examples
- Adding new file protection features

## Context
From a real scenario where:
- A CSS migration caused small visual regression test failures
- Claude attempted to add dimension tolerance (±10px) to make tests pass
- The `uneditableFiles: ["visual-regression/**"]` pattern blocked the modification
- This preserved test integrity by preventing a "make tests pass" approach that would weaken the test suite

## User Impact
**Documentation readers** will see a concrete example of file protection in action, understanding not just the mechanics but the *purpose* of protecting files to enforce project values.

## Success Criteria
- [ ] README.md includes the file protection example in "Real-World Scenarios"
- [ ] docs/src/content/docs/guides/hooks.md includes the example in PreToolUse section
- [ ] Example includes the actual error output showing pattern matching
- [ ] Example explains the *why* (preserving test integrity) not just the *what* (blocked edit)

## Dependencies
None - this is purely documentation.

## Migration Notes
N/A - no breaking changes, only documentation additions.

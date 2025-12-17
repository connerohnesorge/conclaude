# Design: Document File Protection Example

## Overview
This change adds a real-world example of file protection to both README.md and the documentation site, showing how `uneditableFiles` patterns enforce project values beyond simple "don't modify this file" use cases.

## Approach

### 1. README.md Addition
Add a new scenario to the "Real-World Scenarios" section showing:
- The context (visual regression test integrity)
- Claude's attempted action (adding tolerance to failing tests)
- conclaude's intervention (blocking the edit with clear error)
- The outcome (test integrity preserved)

**Location:** After "Scenario 2: The 'Where Did This File Come From?' Mystery" and before "Scenario 3: The 'Continuous Refactoring' Workflow"

### 2. Documentation Site Addition
Enhance the PreToolUse hook documentation in the guides section with:
- Expanded "Example Scenario" showing the visual regression case
- Actual error output from conclaude
- Explanation of how the pattern matching works

**Location:** `docs/src/content/docs/guides/hooks.md` in the PreToolUse section

## Example Content Structure

### README.md Scenario Format:
```markdown
### Scenario: The "Test Integrity Enforcer"

**The Problem:**
Your visual regression tests caught layout shifts after a CSS migration. Claude suggests "adding a small tolerance (±10px)" to make the tests pass—but that defeats the purpose of pixel-perfect visual testing.

**The conclaude Solution:**
[Configuration showing uneditableFiles: ["visual-regression/**"]]

**What Happens:**
[Show the actual error output with pattern matching]

**The Result:**
Claude can't weaken your tests. Instead, it must fix the actual CSS issues.
```

### docs/hooks.md Enhancement:
Expand the existing "Example Scenario" with:
```markdown
**Real-World Example: Protecting Test Integrity**

[Show the same scenario with code block formatting]
```

## Rationale

### Why This Example?
1. **Demonstrates value enforcement**: Not just "protect files" but "protect *what files represent*"
2. **Shows error messages**: Helps users understand what they'll see when rules trigger
3. **Relatable scenario**: Many projects have visual regression tests or similar integrity-critical code
4. **Concrete over abstract**: Real bash output is more compelling than hypothetical examples

### Why These Locations?
1. **README.md "Real-World Scenarios"**: Already contains scenario-based examples; fits the pattern
2. **hooks.md PreToolUse section**: Where users learn about file protection; natural place for deep-dive example

## Trade-offs

### Considered Alternatives
1. **Create a dedicated "Examples" page**: Decided against to avoid fragmentation; scenarios work better inline
2. **Add to configuration reference**: Too technical; examples work better in conceptual documentation
3. **Create a "Use Cases" guide**: Premature; can extract later if we accumulate more examples

### Chosen Approach Benefits
- Minimal disruption to existing docs structure
- Enhances existing sections rather than creating new ones
- Makes file protection more tangible immediately where users learn about it

## Implementation Notes

### Content Guidelines
- Keep bash error output formatted as code blocks for readability
- Explain the pattern `visual-regression/**` explicitly
- Connect the technical mechanism (pattern blocking) to the business value (test integrity)
- Use active voice and concrete language

### Validation
- Read through both additions in context to ensure flow
- Verify code blocks render correctly (test in docs preview if available)
- Check that the scenario doesn't contradict any existing content

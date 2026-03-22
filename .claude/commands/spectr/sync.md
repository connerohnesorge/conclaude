---
description: Detect spec drift from code and update specs interactively.
hooks:
  PreToolUse:
  - matcher: ''
    hooks:
    - type: command
      command: conclaude Hooks PreToolUse --skill sync
      timeout: 600
  PostToolUse:
  - matcher: ''
    hooks:
    - type: command
      command: conclaude Hooks PostToolUse --skill sync
      timeout: 600
  Stop:
  - hooks:
    - type: command
      command: conclaude Hooks Stop --skill sync
      timeout: 600
  SessionStart:
  - hooks:
    - type: command
      command: conclaude Hooks SessionStart --skill sync
      timeout: 600
  SessionEnd:
  - hooks:
    - type: command
      command: conclaude Hooks SessionEnd --skill sync
      timeout: 600
  Notification:
  - matcher: ''
    hooks:
    - type: command
      command: conclaude Hooks Notification --skill sync
      timeout: 600
  PreCompact:
  - hooks:
    - type: command
      command: conclaude Hooks PreCompact --skill sync
      timeout: 600
  PermissionRequest:
  - matcher: ''
    hooks:
    - type: command
      command: conclaude Hooks PermissionRequest --skill sync
      timeout: 600
  UserPromptSubmit:
  - matcher: ''
    hooks:
    - type: command
      command: conclaude Hooks UserPromptSubmit --skill sync
      timeout: 600
---

<!-- spectr:START -->

### Guardrails
- Code is the source of truth—specs should reflect actual implementation.
- Only update specs after user confirms each change.
- Keep spec updates minimal and focused on actual drift.
- Create missing specs for new capabilities supported by the implementation.
- Refer to `spectr/AGENTS.md` for Spectr conventions.

### Steps
1. Determine scope:
   - If arguments specify capabilities (e.g., `<Capabilities>auth, cli</Capabilities>`), focus on those.
   - Otherwise, ask the user which capabilities to sync, or analyze recent git changes to suggest candidates.

2. Load current specs:
   - Read the `spectr/specs/` directory to enumerate capabilities.
   - Read each relevant `spectr/specs/<capability>/spec.md`.

3. Analyze implementation:
   - For each capability, identify related code (by directory name, imports, or user guidance).
   - Read the implementation and understand actual behavior.

4. Detect drift:
   - Compare code behavior against spec requirements and scenarios.
   - Identify: new features (to be ADDED), changed behavior (to be MODIFIED), removed features (to be REMOVED).

5. Interactive review:
   - For each drift item, present: the requirement, what code does, and proposed spec change.
   - Ask user to confirm, modify, or skip each change.

6. Apply updates:
   - Edit specs directly with confirmed changes.
   - Run `spectr validate --strict` to ensure validity.
   - Show summary of changes made.

### Reference
- Read `spectr/specs/<capability>/spec.md` to view current spec content.
- Search code with `rg` to find implementations.
- Validate after edits with `spectr validate --strict`.


<!-- spectr:END -->


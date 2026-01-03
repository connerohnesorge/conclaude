---
name: spectr-accept-wo-spectr-bin
description: |
  Accept Spectr change proposals by converting tasks.md to tasks.jsonc without requiring the spectr binary.
  USE WHEN you're in a sandboxed or restricted execution context and spectr is not available in your path.
  DO NOT USE WHEN you need a lightweight alternative for task acceptance, but have the spectr binary available.
  DO NOT USE when you have the spectr binary available.
compatibility:
  requirements:
    - jq (JSON processor)
  platforms:
    - Linux
    - macOS
    - Unix-like systems with bash
---

# Spectr Accept (Without Binary)

This skill provides the ability to accept Spectr change proposals by converting
`tasks.md` files into `tasks.jsonc` format without requiring the `spectr`
binary. This is particularly useful in sandboxed environments, CI pipelines, or
fresh repository checkouts where the spectr binary may not be available.

## Usage

The skill provides a `scripts/accept.sh` script that converts task markdown
files into the JSONC format expected by Spectr.

### Basic Usage

```bash
bash .claude/skills/spectr-accept-wo-spectr-bin/scripts/accept.sh <change-id>
```

### Example

```bash
# Accept a change with ID "add-new-feature"
bash .claude/skills/spectr-accept-wo-spectr-bin/scripts/accept.sh add-new-feature
```

This will:
1. Read `spectr/changes/add-new-feature/tasks.md`
2. Parse the markdown task lists
3. Generate `spectr/changes/add-new-feature/tasks.jsonc`

## Requirements

The accept script requires `jq` to be installed for JSON generation. You can
verify jq is available:

```bash
which jq
```

## Task Format

The script expects tasks in markdown format with the following structure:

```markdown
# Section Name

## Tasks

- [ ] 1.1 Task description here
- [x] 1.2 Completed task description
```

Tasks can be:
- `[ ]` - Pending (not started)
- `[x]` - Completed

## Output Format

The script generates a `tasks.jsonc` file with the following structure:

```jsonc
{
  "version": 1,
  "tasks": [
    {
      "id": "1.1",
      "section": "Section Name",
      "description": "Task description here",
      "status": "pending"
    },
    {
      "id": "1.2",
      "section": "Section Name",
      "description": "Completed task description",
      "status": "completed"
    }
  ]
}
```

## Limitations

- This script provides basic task conversion functionality
- For advanced features like hierarchical task validation, use the full
  `spectr accept` command
- The script does not validate task ID uniqueness or proper sequencing
- Complex markdown formatting in task descriptions may not be preserved

## When to Use

Use this skill when:
- The spectr binary is not available in your environment
- You're in a sandboxed or restricted execution context
- You need a lightweight alternative for task acceptance
- You're running in CI/CD pipelines without spectr installed

For production workflows with validation requirements, consider installing the
full spectr binary.

# conclaude
<img src="https://github.com/connerohnesorge/conclaude/blob/main/docs/src/assets/logo.png" alt="Logo" width="105">

**The guardrails your Claude Code sessions need.**

A high-performance Rust CLI tool that provides essential guardrails for Claude Code sessions through a comprehensive hook system. Conclaude ensures your AI coding sessions maintain code quality, follow project standards, and respect your development workflows.

<!-- [![Crates.io](https://img.shields.io/crates/v/conclaude)](https://crates.io/crates/conclaude) -->
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/connerohnesorge/conclaude/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/connerohnesorge/conclaude/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/connerohnesorge/conclaude?display_name=tag&sort=semver)](https://github.com/connerohnesorge/conclaude/releases)


Born from real developer frustration, conclaude transforms chaotic AI coding sessions into controlled, validated workflows. It's not just another CLI tool—it's your project's guardian, ensuring that every Claude Code session respects your standards, follows your rules, and maintains your code quality.

## Releases

Official builds are published on GitHub Releases when tags matching `v*` are pushed.

- Latest release: https://github.com/connerohnesorge/conclaude/releases/latest
- All releases: https://github.com/connerohnesorge/conclaude/releases

### Quick Install Options

#### Option 1: Shell Script (Recommended)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.sh | sh
```

#### Option 2: PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.ps1 | iex"
```

#### Option 3: NPM Package

```bash
npm install conclaude
```

#### Option 4: Manual Binary Download

```bash
# Linux x86_64 example
# Visit https://github.com/connerohnesorge/conclaude/releases/latest to find the download URL for your platform
curl -L -o conclaude.tar.xz \
  https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-x86_64-unknown-linux-gnu.tar.xz
tar -xf conclaude.tar.xz
chmod +x conclaude && sudo mv conclaude /usr/local/bin/
conclaude --version
```

**Available platforms:** Apple Silicon macOS, Intel macOS, x64 Windows, ARM64 Linux, x64 Linux, x64 MUSL Linux.

## The Problem We Solve

AI-assisted coding is revolutionary, but it comes with a challenge: **How do you maintain control and quality standards when an AI is making rapid changes to your codebase?**

Without guardrails, Claude Code sessions can:
- Break existing linting and formatting rules
- Create files in wrong locations
- Skip essential validation steps
- Leave your project in an inconsistent state
- Bypass your carefully crafted development workflows

**conclaude changes this story.**

## How conclaude Changes Everything

Imagine starting every Claude Code session knowing that:

- **Your linting rules will be respected** - No more broken formatting or style violations
- **Your tests must pass** - Sessions only complete when your test suite is green
- **Your files stay organized** - No mysterious root-level files cluttering your project
- **Your workflows are enforced** - Build processes, validation, and quality checks run automatically
- **Everything is configurably logged** - Complete visibility into what Claude did during each session  

This isn't just wishful thinking—it's what conclaude delivers every single time.

## What Makes conclaude Different

While other tools try to bolt-on AI safety as an afterthought, conclaude was built from the ground up specifically for Claude Code workflows. Here's what sets it apart:

- **Purpose-Built for Claude Code**: Native integration with Claude's lifecycle hooks—no hacks, no workarounds
- **Zero Configuration Friction**: Simple YAML config that just works, with automatic directory tree search
- **Fail-Fast Protection**: Catches problems immediately, not after damage is done
- **Extensible Hook System**: Handle PreToolUse, PostToolUse, Stop, and more lifecycle events
- **Session-Aware Logging**: Every action is tracked with session context for complete auditability  

## Core Capabilities

- **Comprehensive Hook System**: Handle all Claude Code lifecycle events with precision
- **YAML Configuration**: Human-readable configuration that scales with your team
- **Command Execution**: Run your existing lint, test, and build commands automatically
- **File Protection**: Prevent unwanted file creation with intelligent pattern matching
- **Session Logging**: Winston-based logging with session-specific file output
- **Infinite Monitoring**: Optional continuous monitoring for long-running development sessions

## Real-World Scenarios

### Scenario 1: The "Oops, My Tests Are Broken" Prevention

**Before conclaude:**
```
Developer: "Claude, add user authentication to my app"
Claude: *writes beautiful auth code*
Developer: *tries to deploy*
CI/CD: FAILED - 47 test failures, linting errors everywhere
Developer: *spends 2 hours fixing what should have been caught*
```

**With conclaude:**
```
Developer: "Claude, add user authentication to my app"
Claude: *writes beautiful auth code*
conclaude: SUCCESS - All tests pass, linting clean
Developer: *deploys confidently*
```

### Scenario 2: The "Where Did This File Come From?" Mystery

**The Problem:**
You're reviewing Claude's work and find `config.json`, `temp.js`, and `debug.log` scattered in your project root. Your clean directory structure is now a mess.

**The conclaude Solution:**
```yaml
# .conclaude.yaml
preToolUse:
  preventRootAdditions: true  # No more mystery files!
```

Claude tries to create a root file → conclaude blocks it → Claude puts it in the right place.

### Scenario 3: The "Continuous Refactoring" Workflow

**The Vision:**
You're pair programming with Claude for hours, making incremental improvements. You want validation after every change, not just at the end.

**The Setup:**
```yaml
# .conclaude.yaml
stop:
  infinite: true
  commands:
    - run: bun x tsc --noEmit
    - run: bun test --silent
  infiniteMessage: "Monitoring active - your code stays clean!"
```

Now every small change gets validated immediately. No surprises at the end of a long session.

## Installation

### From Crates.io (Recommended)

```bash
# Install from crates.io
cargo install conclaude
```

### From Source

```bash
# Clone and build from source
git clone https://github.com/connerohnesorge/conclaude.git
cd conclaude
cargo build --release

# The binary will be in target/release/conclaude
```

### Nix Flake Installation

```bash
# Use the flake directly
nix run github:connerohnesorge/conclaude -- --help
```

#### Adding conclaude to your development shell

Add conclaude as a flake input and include it in your development shell:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    conclaude.url = "github:connerohnesorge/conclaude";
  };

  outputs = { self, nixpkgs, conclaude, ... }:
    let
      system = "x86_64-linux"; # or your system
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.default = pkgs.mkShell {
        packages = [
          conclaude.packages.${system}.default
          # your other packages...
        ];
        
        shellHook = ''
          echo "conclaude available in development environment"
          conclaude --help
        '';
      };
    };
}
```

Then enter the development shell:

```bash
nix develop
```

### Development Setup

```bash
# Clone the repository
git clone https://github.com/connerohnesorge/conclaude.git
cd conclaude

# Build the project
cargo build

# Run tests
cargo test

# Install for development
cargo install --path .
```

## Configuration: Your Project's Rulebook

Think of conclaude's configuration as your project's constitution—the fundamental rules that govern how Claude Code can interact with your codebase. It's designed to be simple enough to set up in minutes, yet powerful enough to handle complex enterprise workflows.

### How Configuration Works

conclaude finds your rules automatically by searching up the directory tree, looking for:

1. `.conclaude.yaml` - Your main configuration file (recommended)
2. `.conclaude.yml` - Alternative YAML extension

The search starts from the current directory and moves up through parent directories until a configuration file is found, the filesystem root is reached, or the maximum search depth (12 levels) is exceeded.

No complex setup, no environment variables to manage. Just drop a `.conclaude.yaml` file in your project (or any parent directory) and you're protected.

> **Note on Configuration Changes**: As of recent versions, the configuration structure has been consolidated. The former `rules` section has been merged into `preToolUse` for better organization. If you're upgrading from an older version, update your configuration file by renaming `rules:` to `preToolUse:`. All file protection and validation features remain unchanged—only the section name has changed.

### Your First Configuration

Here's what a real-world configuration looks like:

```yaml
# .conclaude.yaml - Your project's guardrails

# Commands that MUST pass before any session ends
stop:
  commands:
    - run: cargo fmt --check
      message: "Code formatting check"
    - run: cargo clippy -- -D warnings
      message: "Linting with warnings as errors"
    - run: cargo test
      message: "All tests must pass"
    
    # Skill-specific validation
    - run: cargo test --lib
      skill: "tester"
      message: "Library tests must pass for tester skill"

# File protection rules
preToolUse:
  preventRootAdditions: true    # Keep project root clean
  uneditableFiles:              # Protect critical files
    - "Cargo.toml"              # Don't modify package manifest
    - "Cargo.lock"              # Lock file is sacred
    - ".env*"                   # Secrets stay secret
    
    # Skill-specific protection
    - pattern: "src/**/*.rs"
      skill: "doc*"
      message: "Documentation skills should not modify source code"
```

**What this accomplishes:**
- Claude can't break your Rust compilation
- All tests must pass before session completion
- Your formatting and linting rules are automatically enforced
- No surprise files cluttering your project root
- Critical configuration files stay untouched

### Advanced Scenarios

#### Continuous Monitoring During Long Sessions
```yaml
# Perfect for refactoring sessions or long pair-programming
stop:
  commands:
    - run: cargo check --quiet
      message: "Fast compilation check"
    - run: cargo test --quiet
      message: "Silent test execution"
  infinite: true           # Validate after every change
  infiniteMessage: "Watching your code quality..."
```

#### Enterprise-Grade Protection
```yaml
# Maximum security for production codebases
stop:
  commands:
    - run: cargo audit
      message: "Security audit"
    - run: cargo fmt --check
      message: "Strict formatting"
    - run: cargo clippy -- -D warnings
      message: "All clippy warnings as errors"
    - run: cargo test --all
      message: "Test all packages"
    - run: cargo build --release
      message: "Release build"

preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "Cargo.toml"
    - "Cargo.lock"
    - ".env*"
    - "target/**"
    - ".github/workflows/**"
    - "src/lib.rs"            # Protect main library entry point
```

### Configuration Reference

The complete configuration schema is defined as Rust structs with serde serialization. Key sections include:

- **stop**: Commands and settings for session termination hooks
- **preToolUse**: File protection, validation rules, and pre-execution controls
- **notifications**: System notification settings for hook events

The JSON schema for IDE autocomplete and validation is automatically published with each release at:
`https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-schema.json`

Developers can regenerate the schema locally using: `cargo run --bin generate-schema`

#### System Notifications

conclaude can send system notifications when hooks execute, providing real-time feedback about what's happening in your Claude Code sessions. This is especially useful during long-running tasks or when working in multiple windows.

```yaml
# Enable notifications for specific hooks
notifications:
  enabled: true                    # Turn on notifications
  hooks: ["Stop", "PreToolUse"]    # Which hooks trigger notifications
```

**Available hook names:**
- `"Stop"` - When the stop hook runs (tests, linting, etc.)
- `"PreToolUse"` - Before tools execute (file edits, commands, etc.)
- `"PostToolUse"` - After tools complete successfully
- `"SessionStart"` - When a new Claude session begins
- `"UserPromptSubmit"` - When you submit input to Claude
- `"Notification"` - When Claude sends internal notifications
- `"SubagentStart"` - When Claude subagents begin their tasks
- `"SubagentStop"` - When Claude subagents complete tasks
- `"PreCompact"` - Before transcript compaction

**Wildcards:**
Use `["*"]` to receive notifications for all hooks:

```yaml
notifications:
  enabled: true
  hooks: ["*"]  # All hooks will trigger notifications
```

**Notification content:**
- **Title**: "Conclaude - [HookName]"
- **Body**: Context-specific information about the hook execution
- **Urgency**: Critical for failures, Normal for successes

**Example:**
```yaml
# Get notified when tests run or when files are blocked
notifications:
  enabled: true
  hooks:
    - "Stop"           # Know when tests/linting complete
    - "PreToolUse"     # Know when file operations are blocked
```

## Understanding the Hook System

conclaude taps into Claude Code's lifecycle through strategic intervention points called "hooks." Think of hooks as security checkpoints in your development workflow—each one serves a specific purpose in keeping your codebase safe and consistent.

### The Three Critical Moments

#### PreToolUse Hook: The Gatekeeper
*Fired the moment before Claude tries to use any tool*

**What it protects against:**
- Claude creating files in your project root (when you prefer organized subdirectories)
- Modifications to protected files like `package.json` or `.env` files
- Any tool usage that violates your project's rules

**Real example:** Claude wants to create `debug.log` in your project root, but your `preventRootAdditions` rule blocks it. Claude adapts and creates `logs/debug.log` instead.

#### PostToolUse Hook: The Observer
*Fired immediately after Claude completes any tool operation*

**What it enables:**
- Complete audit trail of every change Claude makes
- Performance monitoring (how long did that operation take?)
- Session-specific logging with full context
- Post-processing and validation of tool results

**Real example:** After Claude edits a file, PostToolUse logs exactly what changed, when, and in which session—giving you complete traceability.

#### Stop Hook: The Validator (Most Important)
*Fired when Claude thinks the session is complete*

**This is where the magic happens.** The Stop hook is your last line of defense and your quality assurance engine:

- **Runs your validation commands** (lint, test, build, etc.)
- **Blocks session completion** if any check fails  
- **Forces Claude to fix issues** before you see "success"
- **Optional infinite mode** for continuous validation during long sessions

**Real example:** Claude finishes implementing a feature. Stop hook runs your tests, finds 3 failures, blocks completion. Claude sees the errors and fixes them automatically. Only then does the session complete successfully.

### Slash Commands and Skills Support

Claude Code supports organizing reusable capabilities as **Slash Commands** (in `.claude/commands/*.md`) and **Skills** (in `.claude/skills/*.md`). conclaude can inject hooks directly into these files, allowing you to enforce guardrails specific to particular commands or skills.

By using the `--skill` flag, conclaude handlers can distinguish between different skills and apply targeted rules.

#### Key Features:
- **Automatic Injection**: `conclaude init` discovers and injects hooks into all command and skill files.
- **Skill Scoping**: Target rules to specific skills or skill patterns (e.g., `skill: "test*"`).
- **Environment Variable**: Hook handlers receive the current skill name via `CONCLAUDE_SKILL`.

### Supporting Cast of Hooks

- **UserPromptSubmit** - Intercept and potentially modify your prompts before Claude sees them
- **SessionStart** - Initialize logging, set up monitoring, prepare your workspace
- **SubagentStart** - Track when Claude's internal subprocesses begin their work
- **SubagentStop** - Handle completion of Claude's internal subprocesses
- **Notification** - Process and potentially filter system notifications
- **PreCompact** - Prepare transcripts before they're compressed or archived

## Configuration Examples

### Basic Configuration (`.conclaude.yaml`)

```yaml
# Commands to run during Stop hook
stop:
  commands:
    - run: cargo check
    - run: cargo test
    - run: cargo build

# Validation rules
preToolUse:
  # Block file creation at repository root
  preventRootAdditions: true

  # Files that cannot be edited (glob patterns)
  uneditableFiles:
    - "Cargo.toml"
    - "Cargo.lock"
    - ".env*"
    - "target/**"
```

### Development Configuration Example

```yaml
# Minimal checks for development
stop:
  commands:
    - run: echo "Running development checks..."
    - run: cargo check

preToolUse:
  preventRootAdditions: false  # Allow root edits during development
  uneditableFiles:
    - "Cargo.toml"  # Still protect Cargo.toml
```

### Production Configuration Example

```yaml
# Comprehensive validation for production
stop:
  commands:
    - run: echo "Running production validation..."
    - run: cargo fmt --check
    - run: cargo clippy -- -D warnings
    - run: cargo test
    - run: cargo build --release

preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "Cargo.toml"
    - "Cargo.lock"
    - ".env*"
    - "target/**"
```

### Infinite Mode Configuration Example

```yaml
# Continuous monitoring with infinite mode
stop:
  commands:
    - run: echo "Starting continuous monitoring..."
    - run: cargo check
    - run: cargo test
  infinite: true
  infiniteMessage: "Monitoring active - press Ctrl+C to stop"

preToolUse:
  preventRootAdditions: false  # Allow file changes during development
  uneditableFiles:
    - "Cargo.toml"
```

## Usage

### Claude Code Integration

conclaude is designed to be used as a hook handler in Claude Code. After global installation, use the `conclaude init` command to automatically configure Claude Code hooks:

```bash
# Initialize conclaude in your project
conclaude init
```

This creates:
- `.conclaude.yaml` - Your project configuration
- `.claude/settings.json` - Claude Code hook configuration

#### Manual Claude Code Configuration

If you prefer manual setup, configure hooks in your Claude Code settings:

> **Note**: As of the current version, all hook commands are nested under the `Hooks` parent subcommand (e.g., `conclaude Hooks PreToolUse` instead of `conclaude PreToolUse`).

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "conclaude Hooks PreToolUse"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "conclaude Hooks PostToolUse"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "conclaude Hooks Stop"
          }
        ]
      }
    ]
  }
}
```

### Initialize Configuration

```bash
# Create initial .conclaude.yaml configuration
conclaude init

# Force overwrite existing configuration
conclaude init --force

# Specify custom paths
conclaude init --config-path ./custom.yaml --claude-path ./.claude-custom
```

### Validate Configuration

The `validate` command checks your conclaude configuration file for syntax errors and schema compliance without running any hooks. This is especially useful for:

- Verifying configuration changes before committing
- CI/CD pipeline validation
- Pre-deployment configuration checks
- Troubleshooting configuration issues

```bash
# Validate default configuration file (.conclaude.yaml)
conclaude validate

# Validate a specific configuration file
conclaude validate --config-path ./config/production.yaml

# Use in scripts with exit code checking
conclaude validate && echo "Config is valid" || echo "Config has errors"
```

**Exit Codes:**
- **0**: Configuration is valid and can be loaded successfully
- **Non-zero**: Configuration has syntax errors, schema violations, or cannot be found

**What gets validated:**
- YAML syntax correctness
- Configuration schema compliance
- File path references (checks if uneditable files patterns are valid)
- Command syntax in stop hooks
- Glob pattern validity in rules

**Example output for valid configuration:**
```
Configuration validation successful
Configuration file: /path/to/project/.conclaude.yaml
```

**Example output for invalid configuration:**
```
Error: Configuration validation failed
Reason: Invalid YAML syntax at line 15: unexpected key 'invalid_field'
```

### Manual Testing

```bash
# Test Stop hook
echo '{"session_id":"test","transcript_path":"/tmp/test.jsonl","hook_event_name":"Stop","stop_hook_active":true}' | \
  conclaude Hooks Stop

# Test PreToolUse hook
echo '{"session_id":"test","transcript_path":"/tmp/test.jsonl","hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"test.txt"}}' | \
  conclaude Hooks PreToolUse

# Test SubagentStart hook
echo '{"session_id":"test","transcript_path":"/tmp/test.jsonl","hook_event_name":"SubagentStart","cwd":"/tmp","permission_mode":"ask","agent_id":"coder","subagent_type":"implementation","agent_transcript_path":"/tmp/agent_coder.jsonl"}' | \
  conclaude Hooks SubagentStart

# Test SubagentStop hook
echo '{"session_id":"test","transcript_path":"/tmp/test.jsonl","hook_event_name":"SubagentStop","stop_hook_active":true,"agent_id":"coder","agent_transcript_path":"/tmp/agent_coder.jsonl"}' | \
  conclaude Hooks SubagentStop

# Get help
conclaude --help
conclaude Hooks --help
conclaude Hooks Stop --help
```

### Exit Codes

- **0**: Success - operation allowed to proceed
- **1**: Error - validation failure, parsing error, or handler crash
- **2**: Blocked - hook explicitly blocked the operation

## Hook Behavior Examples

### Stop Hook Command Execution

The Stop hook executes commands from `config.stop.commands` sequentially:

```bash
# Configuration
stop:
  commands:
    - run: cargo check
    - run: cargo test

# Execution: If any command fails, the entire hook fails and blocks the session
OK Command 1/2: cargo check
FAILED Command 2/2: cargo test (exit code 1)
BLOCKED: Command failed with exit code 1: cargo test
```

### PreToolUse Root Protection

When `preventRootAdditions: true`, file creation is blocked at repo root while edits to existing files are allowed:

```bash
# Blocked operations (creating NEW files at root)
Write → /repo/newfile.txt          BLOCKED (file doesn't exist)
Write → /repo/debug.log            BLOCKED (file doesn't exist)

# Allowed operations (editing EXISTING files at root)
Write → /repo/Cargo.toml           ALLOWED (file exists - modification)
Edit → /repo/package.json          ALLOWED (Edit tool always works)
Write → /repo/.gitignore           ALLOWED (file exists - modification)

# Always allowed operations
Write → /repo/src/component.rs     ALLOWED (subdirectory - not at root)
Read → /repo/Cargo.toml            ALLOWED (read-only operation)
```

**Key insight**: The rule is "prevent root *additions*" (new files), not "prevent root *modifications*" (existing files). This allows you to edit configuration files at the root while still preventing accidental file clutter.

### SubagentStart Hook Payload

> **Note:** Agent-specific hooks are now defined in agent frontmatter (`.claude/agents/*.md`) with the `--agent` flag for cleaner agent-aware execution. Run `conclaude init` to automatically inject hooks into your agent files. This eliminates the need for transcript parsing and provides direct agent context.

The SubagentStart hook is triggered when Claude's internal subagents (like the coder, tester, or stuck agents) begin their work. The hook receives a JSON payload containing information about which subagent is starting and where its transcript will be located.

**SubagentStart Payload Structure:**

```json
{
  "session_id": "abc123def456",
  "transcript_path": "/tmp/conclaude-sessions/main_transcript.jsonl",
  "hook_event_name": "SubagentStart",
  "cwd": "/home/user/project",
  "permission_mode": "ask",
  "agent_id": "coder",
  "subagent_type": "implementation",
  "agent_transcript_path": "/tmp/conclaude-sessions/agent_coder_transcript.jsonl"
}
```

**Payload Fields:**

- `session_id`: Unique identifier for the current Claude Code session
- `transcript_path`: Path to the main session transcript file
- `hook_event_name`: Always "SubagentStart" for this hook
- `cwd`: Current working directory where the session is executing
- `permission_mode`: Permission mode for the session (e.g., "ask", "allow", "deny")
- `agent_id`: **Identifier for the subagent starting** (e.g., "coder", "tester", "stuck", or other agent names)
- `subagent_type`: **Type/category of the subagent** (e.g., "implementation", "testing", "stuck")
- `agent_transcript_path`: **Path to the subagent's transcript file**, allowing monitoring of the agent's work as it progresses

**Use Cases:**

- **Logging and Monitoring**: Track which subagents start and when
- **Initialization**: Perform setup operations when specific subagents begin
- **Metrics**: Collect timing and performance metrics for subagent workflows
- **Integration**: Trigger external systems when specific subagents start
- **Resource Management**: Allocate resources based on subagent type

**Environment Variables Available:**

When SubagentStart hook commands execute, these environment variables are available:

```bash
# Main session context
$CONCLAUDE_SESSION_ID          # Session ID
$CONCLAUDE_TRANSCRIPT_PATH     # Main transcript path
$CONCLAUDE_CWD                 # Working directory
$CONCLAUDE_HOOK_EVENT          # "SubagentStart"
$CONCLAUDE_PERMISSION_MODE     # Permission mode
$CONCLAUDE_CONFIG_DIR          # Directory containing the config file

# Subagent-specific context
$CONCLAUDE_AGENT_ID            # The subagent identifier
$CONCLAUDE_SUBAGENT_TYPE       # The subagent type/category
$CONCLAUDE_AGENT_TRANSCRIPT_PATH # Path to subagent's transcript
```

**Example Configuration:**

```yaml
# .conclaude.yaml
notifications:
  enabled: true
  hooks:
    - "SubagentStart"  # Get notified when subagents start
```

**Manual Testing:**

```bash
# Test SubagentStart hook with complete payload
cat > subagent_start_payload.json << 'EOF'
{
  "session_id": "test-session-123",
  "transcript_path": "/tmp/main_transcript.jsonl",
  "hook_event_name": "SubagentStart",
  "cwd": "/home/user/project",
  "permission_mode": "ask",
  "agent_id": "coder",
  "subagent_type": "implementation",
  "agent_transcript_path": "/tmp/agent_coder_transcript.jsonl"
}
EOF

# Send to conclaude
cat subagent_start_payload.json | conclaude Hooks SubagentStart
```

### SubagentStop Hook Payload

> **Note:** Agent-specific hooks are now defined in agent frontmatter (`.claude/agents/*.md`) with the `--agent` flag. This approach is cleaner than transcript parsing and provides direct agent context to hook handlers.

The SubagentStop hook is triggered when Claude's internal subagents (like the coder, tester, or stuck agents) complete their work. The hook receives a JSON payload containing information about which subagent completed and where its transcript is located.

**SubagentStop Payload Structure:**

```json
{
  "session_id": "abc123def456",
  "transcript_path": "/tmp/conclaude-sessions/main_transcript.jsonl",
  "hook_event_name": "SubagentStop",
  "cwd": "/home/user/project",
  "stop_hook_active": true,
  "agent_id": "coder",
  "agent_transcript_path": "/tmp/conclaude-sessions/agent_coder_transcript.jsonl"
}
```

**Payload Fields:**

- `session_id`: Unique identifier for the current Claude Code session
- `transcript_path`: Path to the main session transcript file
- `hook_event_name`: Always "SubagentStop" for this hook
- `cwd`: Current working directory where the session is executing
- `stop_hook_active`: Whether stop hook validation is enabled for this session
- `agent_id`: **NEW** - Identifier for the subagent that completed (e.g., "coder", "tester", "stuck", or other agent names)
- `agent_transcript_path`: **NEW** - Path to the subagent's transcript file, allowing access to the agent's work history and decisions

**Use Cases:**

- **Logging and Monitoring**: Track which subagents complete and when
- **Cleanup Operations**: Perform cleanup after specific subagents finish
- **Validation Checks**: Validate work done by subagents using their transcript
- **Integration**: Trigger external systems when specific subagents complete
- **Metrics**: Collect performance metrics per subagent

**Environment Variables Available:**

When SubagentStop hook commands execute, these environment variables are available:

```bash
# Main session context
$CONCLAUDE_SESSION_ID          # Session ID
$CONCLAUDE_TRANSCRIPT_PATH     # Main transcript path
$CONCLAUDE_CWD                 # Working directory
$CONCLAUDE_HOOK_EVENT          # "SubagentStop"
$CONCLAUDE_CONFIG_DIR          # Directory containing the config file

# Subagent-specific context
$CONCLAUDE_AGENT_ID            # The raw agent identifier (e.g., "adb0a8b")
$CONCLAUDE_AGENT_NAME          # The semantic agent name (e.g., "coder", "tester", "stuck")
$CONCLAUDE_AGENT_TRANSCRIPT_PATH # Path to subagent's transcript
```

**Example Configuration:**

```yaml
# .conclaude.yaml
notifications:
  enabled: true
  hooks:
    - "SubagentStop"  # Get notified when subagents complete
```

**Manual Testing:**

```bash
# Test SubagentStop hook with complete payload
cat > subagent_stop_payload.json << 'EOF'
{
  "session_id": "test-session-123",
  "transcript_path": "/tmp/main_transcript.jsonl",
  "hook_event_name": "SubagentStop",
  "cwd": "/home/user/project",
  "stop_hook_active": true,
  "agent_id": "coder",
  "agent_transcript_path": "/tmp/agent_coder_transcript.jsonl"
}
EOF

# Send to conclaude
cat subagent_stop_payload.json | conclaude Hooks SubagentStop
```

## Development

### Commands

```bash
# Format code
cargo fmt

# Run linting
cargo clippy

# Run tests
cargo test

# Build for distribution
cargo build --release

# Run hooks directly (development)
cargo run -- <hook-type>

# Use Nix development environment
nix develop -c lint    # Run linting
nix develop -c tests   # Run tests
```

### Project Structure

```
├── src/
│   ├── main.rs             # Main CLI entry point
│   ├── config.rs           # Configuration loading and parsing
│   ├── types.rs            # Rust type definitions for payloads
│   ├── hooks.rs            # Hook handler implementations
│   ├── logger.rs           # Logging configuration
│   ├── schema.rs           # JSON Schema generation
│   ├── lib.rs              # Library exports
│   └── default-config.yaml # Default configuration template
├── .conclaude.yaml         # YAML configuration file
├── flake.nix               # Nix development environment
├── Cargo.toml              # Rust package manifest
└── README.md
```

### Configuration Loading

Configuration is loaded using native Rust YAML parsing with automatic directory tree search:

1. `.conclaude.yaml` - Primary configuration file
2. `.conclaude.yml` - Alternative YAML extension

The search starts from the current directory and moves up the directory tree until a configuration file is found, the filesystem root is reached, or the maximum search depth (12 levels) is exceeded.

If no configuration file is found, conclaude will display the searched locations and suggest running `conclaude init` to generate a template configuration.

### Adding New Hooks

1. Define payload struct in `src/types.rs`
2. Add handler function in `src/hooks.rs`
3. Create command variant in `src/main.rs`
4. Register command with clap CLI parser

## Architecture

### Hook Processing Flow

```
stdin JSON → read_payload_from_stdin() → validate_base_payload() → handler() → HookResult → exit code
```

### Configuration Resolution

```
.conclaude.yaml → serde_yaml parsing → ConclaudeConfig struct
```

### Command Execution (Stop Hook)

```
config.stop.run → extract_bash_commands() → tokio::process::Command → sequential execution → fail fast
```

## Features

### Hook System

- **PreToolUse**: Block or validate tool usage before execution
- **PostToolUse**: Log and analyze tool results after execution
- **Stop**: Run validation commands before session completion
- **SessionStart**: Initialize session-specific logging and setup
- **UserPromptSubmit**: Process and validate user input
- **Notification**: Handle system notifications and alerts
- **SubagentStart**: Track subagent initialization and setup
- **SubagentStop**: Manage subagent completion events
- **PreCompact**: Handle transcript compaction preparation

### Configuration Features

- **YAML Configuration**: Human-readable configuration with JSON Schema validation
- **File Protection**: Prevent edits to critical files using glob patterns
- **Root Directory Protection**: Keep project root clean from unwanted files
- **Command Validation**: Run custom validation commands (tests, linting, builds)
- **Infinite Mode**: Continuous monitoring for long development sessions
- **Grep Rules**: Content-based validation using pattern matching
- **Tool Usage Validation**: Control which tools can operate on which files

### Performance & Reliability

- **Fast Startup**: Minimal overhead with efficient Rust implementation
- **Configuration Caching**: Avoid repeated file system operations
- **Session Logging**: Comprehensive logging with session-specific output
- **Error Recovery**: Graceful handling of command failures and invalid input
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Command Line Interface

### Available Commands

```bash
# Initialize configuration
conclaude init [--force] [--config-path <path>] [--claude-path <path>]

# Validate configuration
conclaude validate [--config-path <path>]

# Hook handlers (called by Claude Code - all under Hooks subcommand)
conclaude Hooks PreToolUse
conclaude Hooks PostToolUse
conclaude Hooks Stop
conclaude Hooks SessionStart
conclaude Hooks UserPromptSubmit
conclaude Hooks Notification
conclaude Hooks SubagentStart
conclaude Hooks SubagentStop
conclaude Hooks PreCompact
conclaude Hooks PermissionRequest
conclaude Hooks SessionEnd

# Visualize configuration
conclaude visualize [--rule <rule-name>] [--show-matches]

# Global options
--verbose              # Enable debug logging
--disable-file-logging # Disable logging to temporary files
```

## Configuration Reference

### Complete Configuration Schema

```yaml
# Stop hook configuration
stop:
  # Structured commands with custom messages and output control
  commands:
    - run: "cargo check"
      message: "Code compilation check"
      showCommand: true  # Show "Executing command X/Y: cargo check" (default: true)
    - run: "cargo test"
      message: "Tests failed - fix failing tests before continuing"
      showStdout: true
      showCommand: false  # Hide the "Executing command..." line
      maxOutputLines: 50
    - run: "cargo build"
      message: "Build failed - fix compilation errors"
  
  # Infinite mode - continue after successful validation
  infinite: false
  infiniteMessage: "Continue working on the task"

  # Content validation rules
  grepRules:
    - filePattern: "**/*.rs"
      forbiddenPattern: "todo|fixme"
      description: "No TODO or FIXME comments allowed"

### Command Configuration Options

Commands in both `stop` and `subagentStop` hooks support the following fields:

- **run** (required): The command to execute (string)
- **message** (optional): Custom error message shown when command fails (string)
- **showStdout** (optional): Show stdout to user/Claude (boolean, default: false)
- **showStderr** (optional): Show stderr to user/Claude (boolean, default: false)
- **showCommand** (optional): Show "Executing command X/Y: <command>" line (boolean, default: true)
- **maxOutputLines** (optional): Limit output to last N lines, range 1-10000 (number)
- **timeout** (optional): Command timeout in seconds (number)

**Example:**
```yaml
stop:
  commands:
    - run: "npm test"
      showStdout: true       # Show test output
      showStderr: true       # Show test errors
      showCommand: false     # Hide the "Executing command 1/2: npm test" line
      message: "Tests failed - please fix before continuing"
      maxOutputLines: 100    # Only show last 100 lines
      timeout: 300           # Timeout after 5 minutes
```

# File and directory protection rules
preToolUse:
  # Prevent NEW file creation at repository root
  # Existing root files (like Cargo.toml, package.json) can still be edited
  preventRootAdditions: true

  # Files that cannot be edited (glob patterns)
  uneditableFiles:
    - "Cargo.toml"
    - "Cargo.lock"
    - ".env*"
    - "target/**"

  # Tool usage validation
  toolUsageValidation:
    - tool: "Write"
      pattern: "**/*.rs"
      action: "allow"
      message: "Writing to Rust files is allowed"
    - tool: "*"
      pattern: ".env*"
      action: "block"
      message: "Environment files cannot be modified"

### Bash Command Validation

Conclaude can validate Bash commands before execution using glob pattern matching. This allows you to block dangerous commands or create whitelists of allowed commands.

#### Configuration

```yaml
preToolUse:
  toolUsageValidation:
    - tool: "Bash"
      pattern: ""                       # Leave empty when using commandPattern
      commandPattern: "rm -rf /*"       # Glob pattern to match
      matchMode: "full"                 # "full" or "prefix" (defaults to "full")
      action: "block"                   # "block" or "allow"
      message: "Dangerous command blocked"
```

#### Match Modes

**Full Mode** (`matchMode: "full"`)
- The pattern must match the entire command string
- Use for blocking exact dangerous commands
- Default when `matchMode` is omitted

```yaml
commandPattern: "rm -rf /*"
matchMode: "full"
# Matches: rm -rf /
# Matches: rm -rf /tmp
# Does NOT match: sudo rm -rf /     (prefix doesn't match)
# Does NOT match: rm -rf / && echo  (suffix doesn't match)
```

**Prefix Mode** (`matchMode: "prefix"`)
- The pattern must match the beginning of the command
- Use for blocking entire command families
- Matches command start only, not commands in the middle

```yaml
commandPattern: "curl *"
matchMode: "prefix"
# Matches: curl https://example.com
# Matches: curl -X POST https://api.com && echo done
# Does NOT match: echo start && curl https://example.com
```

#### Actions

**Block Action** - Prevents matching commands from executing

```yaml
- tool: "Bash"
  commandPattern: "rm -rf*"
  matchMode: "prefix"
  action: "block"
  message: "Recursive rm commands are not allowed"
```

**Allow Action** - Creates a whitelist where only matching commands are allowed

```yaml
- tool: "Bash"
  commandPattern: "cargo *"
  matchMode: "prefix"
  action: "allow"
  message: "Only cargo commands are permitted"
```

#### Examples

**Block dangerous file operations**
```yaml
- tool: "Bash"
  commandPattern: "rm -rf*"
  matchMode: "prefix"
  action: "block"
  message: "Recursive deletion blocked for safety"
```

**Block force push operations**
```yaml
- tool: "Bash"
  commandPattern: "git push --force*"
  matchMode: "prefix"
  action: "block"
  message: "Force push is not allowed"
```

**Whitelist only safe commands**
```yaml
- tool: "Bash"
  commandPattern: "cargo test*"
  matchMode: "prefix"
  action: "allow"
  message: "Only cargo test commands are allowed in this workflow"

- tool: "Bash"
  commandPattern: "cargo build*"
  matchMode: "prefix"
  action: "allow"
  message: "Only cargo build commands are allowed in this workflow"
```

**Block network requests**
```yaml
- tool: "Bash"
  commandPattern: "curl *"
  matchMode: "prefix"
  action: "block"

- tool: "Bash"
  commandPattern: "wget *"
  matchMode: "prefix"
  action: "block"
```

# Pre-tool-use hook configuration
preToolUse:
  # Content validation before tool execution
  grepRules:
    - filePattern: "**/*.rs"
      forbiddenPattern: "unsafe"
      description: "Unsafe code blocks not allowed"
  
  # Additional directories to protect from additions
  preventAdditions:
    - "docs/"
    - "examples/"

```

### Environment Variables

**General Configuration:**
- `CONCLAUDE_LOG_LEVEL`: Set logging level (debug, info, warn, error)
- `CONCLAUDE_DISABLE_FILE_LOGGING`: Disable logging to temporary files

**Hook Context Variables** (Available to hook commands):
- `CONCLAUDE_SESSION_ID`: Unique session identifier
- `CONCLAUDE_TRANSCRIPT_PATH`: Path to the main session transcript file
- `CONCLAUDE_CWD`: Current working directory where the session is running
- `CONCLAUDE_HOOK_EVENT`: Name of the currently executing hook
- `CONCLAUDE_AGENT`: Current agent name (if applicable, via `--agent`)
- `CONCLAUDE_SKILL`: Current skill/command name (if applicable, via `--skill`)

**SubagentStart Hook Variables** (Available when SubagentStart hook executes):
- `CONCLAUDE_AGENT_ID`: Identifier for the subagent starting (e.g., "coder", "tester", "stuck")
- `CONCLAUDE_SUBAGENT_TYPE`: Type/category of the subagent (e.g., "implementation", "testing", "stuck")
- `CONCLAUDE_AGENT_TRANSCRIPT_PATH`: Path to the subagent's transcript file for monitoring its work
- `CONCLAUDE_PERMISSION_MODE`: Permission mode for the session (e.g., "ask", "allow", "deny")

**SubagentStop Hook Variables** (Available when SubagentStop hook executes):
- `CONCLAUDE_AGENT_ID`: The raw agent identifier (e.g., "adb0a8b")
- `CONCLAUDE_AGENT_NAME`: The semantic agent name extracted from the main transcript (e.g., "coder", "tester", "stuck"). Falls back to `CONCLAUDE_AGENT_ID` if extraction fails.
- `CONCLAUDE_AGENT_TRANSCRIPT_PATH`: Path to the subagent's transcript file for accessing its work history

**Available in all stop hooks:**
- `CONCLAUDE_CONFIG_DIR`: Directory containing the configuration file (commands execute from this directory)

## CI/CD Integration

conclaude's `validate` command is designed to integrate seamlessly into your CI/CD pipelines, ensuring configuration quality before deployment or during pull request validation.

### GitHub Actions

Add configuration validation to your GitHub Actions workflow:

```yaml
name: Validate conclaude Config

on:
  pull_request:
    paths:
      - '.conclaude.yaml'
      - '.conclaude.yml'
  push:
    branches:
      - main

jobs:
  validate-config:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install conclaude
        run: |
          curl --proto '=https' --tlsv1.2 -LsSf \
            https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.sh | sh
          echo "$HOME/.cargo/bin" >> $GITHUB_PATH

      - name: Validate conclaude configuration
        run: conclaude validate

      - name: Validate production config (if exists)
        if: hashFiles('config/production.yaml') != ''
        run: conclaude validate --config-path config/production.yaml
```

### GitLab CI

Add to your `.gitlab-ci.yml`:

```yaml
validate-conclaude:
  stage: test
  image: rust:latest
  before_script:
    - curl --proto '=https' --tlsv1.2 -LsSf
      https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.sh | sh
    - export PATH="$HOME/.cargo/bin:$PATH"
  script:
    - conclaude validate
  only:
    changes:
      - .conclaude.yaml
      - .conclaude.yml
```

### CircleCI

Add to your `.circleci/config.yml`:

```yaml
version: 2.1

jobs:
  validate-config:
    docker:
      - image: cimg/rust:1.70
    steps:
      - checkout
      - run:
          name: Install conclaude
          command: |
            curl --proto '=https' --tlsv1.2 -LsSf \
              https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.sh | sh
            echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> $BASH_ENV
      - run:
          name: Validate configuration
          command: conclaude validate

workflows:
  version: 2
  validate:
    jobs:
      - validate-config
```

### Pre-commit Hook

Use conclaude validation as a git pre-commit hook:

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Check if .conclaude.yaml was modified
if git diff --cached --name-only | grep -q "\.conclaude\.ya\?ml"; then
    echo "Validating conclaude configuration..."
    if ! conclaude validate; then
        echo "Error: conclaude configuration is invalid"
        echo "Please fix the configuration before committing"
        exit 1
    fi
    echo "conclaude configuration is valid"
fi
```

### Docker Container Validation

Validate configuration in a containerized environment:

```dockerfile
FROM rust:1.70-slim as validator

# Install conclaude
RUN curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.sh | sh

# Copy configuration
COPY .conclaude.yaml /app/.conclaude.yaml
WORKDIR /app

# Validate configuration
RUN /root/.cargo/bin/conclaude validate
```

### Make Integration

Add validation to your Makefile:

```makefile
.PHONY: validate-config
validate-config:
	@echo "Validating conclaude configuration..."
	@conclaude validate || (echo "Configuration validation failed" && exit 1)

.PHONY: test
test: validate-config
	@echo "Running tests..."
	@cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/conclaude.git`
3. Create a feature branch: `git checkout -b feature-name`
4. Make your changes and add tests
5. Run the test suite: `cargo test`
6. Run linting: `cargo clippy`
7. Format code: `cargo fmt`
8. Commit your changes: `git commit -am 'Add feature'`
9. Push to the branch: `git push origin feature-name`
10. Submit a Pull Request

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [serde](https://serde.rs/) for JSON/YAML serialization
- CLI powered by [clap](https://clap.rs/)
- Async runtime provided by [tokio](https://tokio.rs/)
- Configuration validation with [schemars](https://docs.rs/schemars/)

---

**Note**: This is the Rust implementation of conclaude. For maximum performance and system integration, the Rust version is recommended over alternative implementations.

// Final test - expecting both workflows to succeed
use anyhow::{Context, Result};
use conclaude_field_derive::FieldList;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Configuration for individual stop commands with optional messages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, FieldList)]
#[serde(deny_unknown_fields)]
pub struct StopCommand {
    /// The shell command to execute
    pub run: String,
    /// Custom error message to display when the command fails (exits with non-zero status)
    #[serde(default)]
    pub message: Option<String>,
    /// Whether to show the command's standard output to the user and Claude. Default: false
    #[serde(default, rename = "showStdout")]
    pub show_stdout: Option<bool>,
    /// Whether to show the command's standard error output to the user and Claude. Default: false
    #[serde(default, rename = "showStderr")]
    pub show_stderr: Option<bool>,
    /// Maximum number of output lines to display (limits both stdout and stderr). Range: 1-10000
    #[serde(default, rename = "maxOutputLines")]
    #[schemars(range(min = 1, max = 10000))]
    pub max_output_lines: Option<u32>,
    /// Optional command timeout in seconds. Range: 1-3600 (1 second to 1 hour). When timeout occurs, the command is terminated and the hook is blocked.
    #[serde(default)]
    #[schemars(range(min = 1, max = 3600))]
    pub timeout: Option<u64>,
}

/// Configuration for individual subagent stop commands with optional messages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, FieldList)]
#[serde(deny_unknown_fields)]
pub struct SubagentStopCommand {
    /// The shell command to execute. Environment variables are available: CONCLAUDE_AGENT_ID, CONCLAUDE_AGENT_TRANSCRIPT_PATH, CONCLAUDE_SESSION_ID, CONCLAUDE_TRANSCRIPT_PATH, CONCLAUDE_HOOK_EVENT, CONCLAUDE_CWD
    pub run: String,
    /// Custom error message to display when the command fails (exits with non-zero status)
    #[serde(default)]
    pub message: Option<String>,
    /// Whether to show the command's standard output to the user and Claude. Default: false
    #[serde(default, rename = "showStdout")]
    pub show_stdout: Option<bool>,
    /// Whether to show the command's standard error output to the user and Claude. Default: false
    #[serde(default, rename = "showStderr")]
    pub show_stderr: Option<bool>,
    /// Maximum number of output lines to display (limits both stdout and stderr). Range: 1-10000
    #[serde(default, rename = "maxOutputLines")]
    #[schemars(range(min = 1, max = 10000))]
    pub max_output_lines: Option<u32>,
    /// Optional command timeout in seconds. Range: 1-3600 (1 second to 1 hour). When timeout occurs, the command is terminated and the hook is blocked.
    #[serde(default)]
    #[schemars(range(min = 1, max = 3600))]
    pub timeout: Option<u64>,
}

/// Configuration for subagent stop hooks with pattern-based command execution.
///
/// This hook allows configuring different commands for different subagent names
/// using pattern matching. Commands run when a subagent finishes its work.
///
/// # Pattern Matching Rules
///
/// - Patterns are matched in the order they appear in the configuration
/// - First matching pattern's commands are executed
/// - Use "*" to match all subagents (put last as fallback)
/// - Glob patterns support: *, ?, \[abc\], \[a-z\], {foo,bar}
///
/// # Environment Variables
///
/// The following environment variables are available in subagent stop commands:
/// - `CONCLAUDE_AGENT_ID` - The subagent's identifier
/// - `CONCLAUDE_AGENT_TRANSCRIPT_PATH` - Path to subagent's transcript
/// - `CONCLAUDE_SESSION_ID` - Current session ID
/// - `CONCLAUDE_TRANSCRIPT_PATH` - Main transcript file path
/// - `CONCLAUDE_HOOK_EVENT` - Always "SubagentStop"
/// - `CONCLAUDE_CWD` - Current working directory
///
/// # Examples
///
/// ```yaml
/// subagentStop:
///   commands:
///     # Exact match - only runs for subagent named "coder"
///     coder:
///       - run: "npm run lint"
///         showStdout: true
///         message: "Linting failed"
///
///     # Glob pattern - runs for any subagent name starting with "test"
///     test*:
///       - run: "npm test"
///         timeout: 600
///
///     # Wildcard - runs for ALL subagents
///     "*":
///       - run: "echo 'Subagent completed'"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, FieldList)]
#[serde(deny_unknown_fields)]
pub struct SubagentStopConfig {
    /// Map of subagent name patterns to command configurations.
    ///
    /// Each key is a glob pattern that matches against the subagent name.
    /// Commands are executed in the order they appear when the pattern matches.
    ///
    /// Pattern examples:
    /// - `"*"` - Matches all subagents (wildcard)
    /// - `"coder"` - Exact match for subagent named "coder"
    /// - `"test*"` - Matches any subagent name starting with "test"
    /// - `"*coder"` - Matches any subagent name ending with "coder"
    ///
    /// Command options (same as stop hook):
    /// - `run`: (required) Command to execute
    /// - `showStdout`: (optional) Show stdout to user/Claude. Default: false
    /// - `showStderr`: (optional) Show stderr to user/Claude. Default: false
    /// - `message`: (optional) Custom error message on non-zero exit
    /// - `maxOutputLines`: (optional) Limit output lines. Range: 1-10000
    /// - `timeout`: (optional) Command timeout in seconds. Range: 1-3600 (1 second to 1 hour). When timeout occurs, command is terminated and hook is blocked.
    #[serde(default)]
    pub commands: std::collections::HashMap<String, Vec<SubagentStopCommand>>,
}

/// Configuration for stop hook commands that run when Claude is about to stop
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, FieldList)]
#[serde(deny_unknown_fields)]
pub struct StopConfig {
    /// List of commands to execute when Claude is about to stop. Commands run in order and can provide custom error messages and control output display.
    #[serde(default)]
    pub commands: Vec<StopCommand>,
    /// Infinite mode - when enabled, allows Claude to continue automatically instead of ending the session after stop hook commands succeed. Default: false
    #[serde(default)]
    pub infinite: bool,
    /// Message to send to Claude when infinite mode is enabled and stop hook commands succeed. Claude receives this message to continue working.
    #[serde(default, rename = "infiniteMessage")]
    pub infinite_message: Option<String>,
}

/// Tool usage validation rule for fine-grained control over tool usage based on file patterns.
///
/// Allows controlling which tools can be used on which files or with which command patterns.
/// Rules are evaluated in order and the first matching rule determines the action.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolUsageRule {
    /// The tool name to match against. Supports glob patterns (e.g., "*" for all tools, "Write", "Bash")
    pub tool: String,
    /// File path pattern to match. Uses glob syntax (e.g., "**/*.js", ".env*")
    pub pattern: String,
    /// Action to take when the rule matches: "allow" or "block"
    pub action: String,
    /// Optional custom message to display when the rule blocks an action
    pub message: Option<String>,
    /// Optional command pattern to match for Bash tool. Uses glob syntax (e.g., "git push --force*", "git *")
    #[serde(rename = "commandPattern")]
    pub command_pattern: Option<String>,
    /// Optional match mode for pattern matching (reserved for future use)
    #[serde(rename = "matchMode")]
    pub match_mode: Option<String>,
}

/// Configuration for an uneditable file rule.
///
/// Files that Claude cannot edit, using glob patterns. Supports various glob patterns
/// for flexible file protection.
///
/// # Formats
///
/// Two formats are supported for backward compatibility:
///
/// 1. **Simple string patterns**: `"*.lock"`
///    - Just the glob pattern as a string
///    - Uses a generic error message when blocking
///
/// 2. **Detailed objects with custom messages**: `{pattern: "*.lock", message: "..."}`
///    - Allows specifying a custom error message
///    - More descriptive feedback when files are blocked
///
/// # Examples
///
/// ```yaml
/// uneditableFiles:
///   # Simple patterns (backward compatible)
///   - "./package.json"      # specific file
///   - "*.md"                # file extension
///   - "src/**/*.ts"         # nested patterns
///   - "docs/**"             # entire directories
///
///   # Detailed patterns with custom error messages
///   - pattern: "*.lock"
///     message: "Lock files are automatically created. Run 'npm install' to update."
///   - pattern: ".env*"
///     message: "Environment files contain secrets. Use .env.example instead."
///   - pattern: "{package,tsconfig}.json"
///     message: "Configuration files require team review before changes."
/// ```
///
/// The `#[serde(untagged)]` attribute allows serde to automatically handle both
/// plain string patterns and detailed object configurations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum UnEditableFileRule {
    /// Detailed format with pattern and optional custom message.
    ///
    /// Allows providing a custom error message that will be shown when Claude
    /// attempts to edit a file matching this pattern.
    #[serde(rename_all = "camelCase")]
    Detailed {
        /// Glob pattern matching files to protect (e.g., "*.lock", ".env*", "src/**/*.ts")
        pattern: String,
        /// Optional custom message to display when blocking edits to matching files
        #[serde(default)]
        message: Option<String>,
    },
    /// Simple format: just a glob pattern string.
    ///
    /// Uses a generic error message when blocking file edits.
    /// Backward compatible with existing configurations.
    Simple(String),
}

impl UnEditableFileRule {
    /// Extract the pattern from either variant
    #[must_use]
    pub fn pattern(&self) -> &str {
        match self {
            UnEditableFileRule::Simple(pattern) => pattern,
            UnEditableFileRule::Detailed { pattern, .. } => pattern,
        }
    }

    /// Get the custom message if present (only from Detailed variant)
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            UnEditableFileRule::Detailed {
                message: Some(msg), ..
            } => Some(msg),
            _ => None,
        }
    }
}

/// Default function that returns true for serde defaults
fn default_true() -> bool {
    true
}

/// Configuration for pre-tool-use hooks that run before tools are executed.
///
/// All file protection rules are consolidated in this section to prevent Claude from
/// making unintended modifications to protected files, directories, or executing
/// dangerous commands.
///
/// # Examples
///
/// ```yaml
/// preToolUse:
///   # Prevent root-level file creation
///   preventRootAdditions: true
///
///   # Protect specific files with glob patterns
///   uneditableFiles:
///     - ".conclaude.yml"
///     - "*.lock"
///     - pattern: ".env*"
///       message: "Environment files contain secrets"
///
///   # Prevent modifications to git-ignored files
///   preventUpdateGitIgnored: false
///
///   # Fine-grained tool control
///   toolUsageValidation:
///     - tool: "Bash"
///       commandPattern: "git push --force*"
///       action: "block"
///       message: "Force push is not allowed"
///
///   # Block additions to specific directories
///   preventAdditions:
///     - "dist"
///     - "build"
///
///   # Protect generated files
///   preventGeneratedFileEdits: true
///   generatedFileMessage: "Cannot modify {file_path} - it contains '{marker}'"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, FieldList)]
#[serde(deny_unknown_fields)]
pub struct PreToolUseConfig {
    /// Directories where file additions are prevented (in addition to root if `preventRootAdditions` is enabled).
    ///
    /// List of directory paths where new files cannot be created. Useful for protecting
    /// build output directories or other generated content.
    ///
    /// # Examples
    ///
    /// ```yaml
    /// preventAdditions:
    ///   - "dist"
    ///   - "build"
    ///   - "node_modules"
    /// ```
    #[serde(default, rename = "preventAdditions")]
    pub prevent_additions: Vec<String>,
    /// Prevent editing of files with generation markers (enabled by default).
    ///
    /// When enabled, checks for common markers like "DO NOT EDIT", "Code generated by",
    /// "@generated", etc. in file contents before allowing edits.
    ///
    /// Default: `true`
    #[serde(default = "default_true", rename = "preventGeneratedFileEdits")]
    pub prevent_generated_file_edits: bool,
    /// Custom message when blocking file edits with generation markers.
    ///
    /// Available placeholders:
    /// - `{file_path}` - The path to the file being blocked
    /// - `{marker}` - The generation marker that was detected
    ///
    /// # Example
    ///
    /// ```yaml
    /// generatedFileMessage: "Cannot modify {file_path} - it contains '{marker}' marker"
    /// ```
    ///
    /// Default: `null` (uses a generic error message)
    #[serde(default, rename = "generatedFileMessage")]
    pub generated_file_message: Option<String>,
    /// Prevent Claude from creating or modifying files at the repository root.
    ///
    /// Helps maintain clean project structure by preventing clutter at the root level.
    /// This is a security best practice to avoid accidental modification of important
    /// configuration files.
    ///
    /// Default: `true`
    #[serde(default = "default_true", rename = "preventRootAdditions")]
    pub prevent_root_additions: bool,
    /// Files that Claude cannot edit, using glob patterns.
    ///
    /// Supports various glob patterns for flexible file protection. By default,
    /// conclaude's own config files are protected to prevent the AI from modifying
    /// guardrail settings - this is a security best practice.
    ///
    /// Supports two formats:
    /// 1. Simple string patterns: `"*.lock"`
    /// 2. Detailed objects with custom messages: `{pattern: "*.lock", message: "..."}`
    ///
    /// # Examples
    ///
    /// ```yaml
    /// uneditableFiles:
    ///   - ".conclaude.yml"    # Protect config
    ///   - ".conclaude.yaml"   # Alternative extension
    ///   - "*.lock"            # Lock files
    ///   - pattern: ".env*"
    ///     message: "Environment files contain secrets. Use .env.example instead."
    /// ```
    ///
    /// Default: `[".conclaude.yml", ".conclaude.yaml"]`
    #[serde(default, rename = "uneditableFiles")]
    pub uneditable_files: Vec<UnEditableFileRule>,
    /// Block Claude from modifying or creating files that match .gitignore patterns.
    ///
    /// When enabled, files matching patterns in .gitignore will be protected.
    /// Uses your existing .gitignore as the source of truth for file protection.
    ///
    /// Default: `false`
    #[serde(default, rename = "preventUpdateGitIgnored")]
    pub prevent_update_git_ignored: bool,
    /// Tool usage validation rules for fine-grained control over tool usage.
    ///
    /// Allows controlling which tools can be used on which files or with which
    /// command patterns. Rules are evaluated in order.
    ///
    /// # Examples
    ///
    /// ```yaml
    /// toolUsageValidation:
    ///   # Allow writing to JavaScript files
    ///   - tool: "Write"
    ///     pattern: "**/*.js"
    ///     action: "allow"
    ///
    ///   # Block environment file modifications
    ///   - tool: "*"
    ///     pattern: ".env*"
    ///     action: "block"
    ///     message: "Environment files cannot be modified"
    ///
    ///   # Block dangerous git operations
    ///   - tool: "Bash"
    ///     commandPattern: "git push --force*"
    ///     action: "block"
    ///     message: "Force push is not allowed"
    /// ```
    ///
    /// Default: `[]` (no validation rules)
    #[serde(default, rename = "toolUsageValidation")]
    pub tool_usage_validation: Vec<ToolUsageRule>,
}

impl Default for PreToolUseConfig {
    fn default() -> Self {
        Self {
            prevent_additions: Vec::new(),
            prevent_generated_file_edits: true,
            generated_file_message: None,
            prevent_root_additions: true,
            uneditable_files: Vec::new(),
            prevent_update_git_ignored: false,
            tool_usage_validation: Vec::new(),
        }
    }
}

/// Configuration for system notifications.
///
/// Controls desktop notifications for hook execution, errors, successes, and system events.
/// Notifications help you stay informed about what conclaude is doing in the background.
///
/// # Examples
///
/// ```yaml
/// # Enable notifications for all hooks
/// notifications:
///   enabled: true
///   hooks: ["*"]
///   showErrors: true
///   showSuccess: true
///   showSystemEvents: true
/// ```
///
/// ```yaml
/// # Enable notifications only for Stop hook
/// notifications:
///   enabled: true
///   hooks: ["Stop"]
///   showErrors: true
///   showSuccess: false
///   showSystemEvents: false
/// ```
///
/// ```yaml
/// # Enable notifications for specific hooks
/// notifications:
///   enabled: true
///   hooks: ["Stop", "PreToolUse"]
///   showErrors: true
///   showSuccess: true
///   showSystemEvents: true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, FieldList)]
#[serde(deny_unknown_fields)]
pub struct NotificationsConfig {
    /// Enable system notifications for hook execution.
    ///
    /// When enabled, conclaude will send desktop notifications based on the configured
    /// notification types (errors, successes, system events) and hook filters.
    ///
    /// Default: `false`
    #[serde(default)]
    pub enabled: bool,
    /// List of hook names that should trigger notifications.
    ///
    /// Use `["*"]` to receive notifications for all hooks, or specify individual hook
    /// names to filter which hooks generate notifications.
    ///
    /// Common hook names:
    /// - `"Stop"` - When Claude is about to stop
    /// - `"PreToolUse"` - Before tools are executed
    /// - `"PostToolUse"` - After tools are executed
    /// - `"SessionStart"` - When a session starts
    /// - `"UserPromptSubmit"` - When user submits a prompt
    /// - `"Notification"` - General notifications
    /// - `"SubagentStop"` - When subagents stop
    /// - `"PreCompact"` - Before transcript compaction
    ///
    /// Examples:
    /// - `["*"]` - All hooks
    /// - `["Stop", "PreToolUse"]` - Only specific hooks
    /// - `["Stop"]` - Only stop hook notifications
    ///
    /// Default: `[]` (no hooks)
    #[serde(default)]
    pub hooks: Vec<String>,
    /// Show error notifications (hook failures, system errors).
    ///
    /// When enabled, you'll receive desktop notifications when hooks fail or system
    /// errors occur. Useful for catching issues early.
    ///
    /// Default: `false`
    #[serde(default, rename = "showErrors")]
    pub show_errors: bool,
    /// Show success notifications (hook completion, successful operations).
    ///
    /// When enabled, you'll receive desktop notifications when hooks complete successfully
    /// and operations finish without errors.
    ///
    /// Default: `false`
    #[serde(default, rename = "showSuccess")]
    pub show_success: bool,
    /// Show system event notifications (session start/end, configuration loaded).
    ///
    /// When enabled, you'll receive desktop notifications for system-level events like
    /// session initialization, configuration loading, and session termination.
    ///
    /// Default: `true`
    #[serde(default = "default_show_system_events", rename = "showSystemEvents")]
    pub show_system_events: bool,
}

/// Configuration for permission request hooks that control tool permission decisions.
///
/// This hook is fired when Claude requests permission to use a tool. Use this to
/// automatically approve or deny tool usage based on configurable rules.
///
/// # Pattern Matching
///
/// Both `allow` and `deny` fields support glob patterns for flexible tool matching:
/// - `"Bash"` - Exact match (only "Bash")
/// - `"*"` - Wildcard (matches any tool)
/// - `"Edit*"` - Prefix match (matches "Edit", "EditFile", etc.)
/// - `"*Read"` - Suffix match (matches "Read", "FileRead", etc.)
///
/// **Important**: Deny patterns take precedence over allow patterns.
///
/// # Security Recommendations
///
/// - **Whitelist approach (recommended)**: Set `default: "deny"` and explicitly list allowed tools
/// - **Blacklist approach (more permissive)**: Set `default: "allow"` and explicitly list denied tools
///
/// # Examples
///
/// ## Whitelist approach (recommended for security)
///
/// ```yaml
/// permissionRequest:
///   default: deny
///   allow:
///     - "Read"       # Allow reading files
///     - "Glob"       # Allow file pattern matching
///     - "Grep"       # Allow content search
///     - "Edit"       # Allow file editing
///     - "Write"      # Allow file writing
///     - "Task"       # Allow subagent tasks
///     - "Bash"       # Allow bash commands
/// ```
///
/// ## Blacklist approach (more permissive)
///
/// ```yaml
/// permissionRequest:
///   default: allow
///   deny:
///     - "BashOutput"   # Block reading background process output
///     - "KillShell"    # Block terminating background shells
/// ```
///
/// ## Mixed approach with patterns
///
/// ```yaml
/// permissionRequest:
///   default: deny
///   allow:
///     - "Read"
///     - "Write"
///     - "Edit*"      # Allow all Edit-based tools
///   deny:
///     - "Bash"       # Explicitly deny even though default is deny
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, FieldList)]
#[serde(deny_unknown_fields)]
pub struct PermissionRequestConfig {
    /// Default decision when a tool doesn't match any allow or deny rule.
    ///
    /// Valid values:
    /// - `"allow"` - Permit tools by default (blacklist approach)
    /// - `"deny"` - Block tools by default (whitelist approach, recommended for security)
    ///
    /// The default action is applied when a tool is requested that doesn't match
    /// any patterns in the `allow` or `deny` lists.
    pub default: String,
    /// Tools to explicitly allow using glob patterns.
    ///
    /// These patterns are checked AFTER deny patterns. If a tool matches both an allow
    /// and a deny pattern, the deny pattern takes precedence.
    ///
    /// # Pattern Examples
    ///
    /// - `"Read"` - Exact match for the Read tool
    /// - `"*"` - Match all tools (use with caution)
    /// - `"Edit*"` - Match any tool starting with "Edit"
    /// - `"*Read"` - Match any tool ending with "Read"
    ///
    /// # Common Tools
    ///
    /// - `"Read"` - Read files
    /// - `"Write"` - Write files
    /// - `"Edit"` - Edit files
    /// - `"Bash"` - Execute bash commands
    /// - `"Glob"` - File pattern matching
    /// - `"Grep"` - Content search
    /// - `"Task"` - Subagent tasks
    ///
    /// Default: `None` (no tools explicitly allowed)
    #[serde(default)]
    pub allow: Option<Vec<String>>,
    /// Tools to explicitly deny using glob patterns.
    ///
    /// Deny patterns take precedence over allow patterns. If a tool matches both
    /// an allow and a deny pattern, it will be denied.
    ///
    /// # Pattern Examples
    ///
    /// - `"BashOutput"` - Block reading background process output
    /// - `"KillShell"` - Block terminating background shells
    /// - `"Bash"` - Block all bash command execution
    /// - `"*"` - Block all tools (use with specific allow rules)
    ///
    /// Default: `None` (no tools explicitly denied)
    #[serde(default)]
    pub deny: Option<Vec<String>>,
}

fn default_show_system_events() -> bool {
    true
}

impl NotificationsConfig {
    /// Check if notifications are enabled for a specific hook
    #[must_use]
    pub fn is_enabled_for(&self, hook_name: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // Check for wildcard
        if self.hooks.iter().any(|hook| hook == "*") {
            return true;
        }

        // Check for specific hook name
        self.hooks.iter().any(|hook| hook == hook_name)
    }
}

/// Main configuration interface matching the TypeScript version
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct ConclaudeConfig {
    #[serde(default)]
    pub stop: StopConfig,
    #[serde(default, rename = "subagentStop")]
    pub subagent_stop: SubagentStopConfig,
    #[serde(default, rename = "preToolUse")]
    pub pre_tool_use: PreToolUseConfig,
    #[serde(default)]
    pub notifications: NotificationsConfig,
    #[serde(default, rename = "permissionRequest")]
    pub permission_request: Option<PermissionRequestConfig>,
}

/// Extract the field name from an unknown field error message
fn extract_unknown_field(error_msg: &str) -> Option<String> {
    // Try to extract the field name from "unknown field `fieldName`"
    if let Some(start) = error_msg.find("unknown field `") {
        let start_idx = start + "unknown field `".len();
        if let Some(end_idx) = error_msg[start_idx..].find('`') {
            return Some(error_msg[start_idx..start_idx + end_idx].to_string());
        }
    }
    None
}

/// Suggest similar field names based on the unknown field
fn suggest_similar_fields(unknown_field: &str, section: &str) -> Vec<String> {
    let all_fields: Vec<(&str, Vec<&str>)> = vec![
        ("stop", StopConfig::field_names()),
        ("subagentStop", SubagentStopConfig::field_names()),
        ("preToolUse", PreToolUseConfig::field_names()),
        ("notifications", NotificationsConfig::field_names()),
        ("permissionRequest", PermissionRequestConfig::field_names()),
        ("commands", StopCommand::field_names()),
        ("subagentStopCommands", SubagentStopCommand::field_names()),
    ];

    // Find the section's valid fields
    let empty_fields: Vec<&str> = vec![];
    let valid_fields = all_fields
        .iter()
        .find(|(s, _)| *s == section)
        .map(|(_, fields)| fields)
        .unwrap_or(&empty_fields);

    // Calculate Levenshtein distance and suggest close matches
    let mut suggestions: Vec<(usize, &str)> = valid_fields
        .iter()
        .map(|field| {
            let distance = levenshtein_distance(unknown_field, field);
            (distance, *field)
        })
        .filter(|(dist, _)| *dist <= 3) // Only suggest if distance is 3 or less
        .collect();

    suggestions.sort_by_key(|(dist, _)| *dist);
    suggestions
        .into_iter()
        .map(|(_, field)| field.to_string())
        .take(3)
        .collect()
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1.eq_ignore_ascii_case(&c2) { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
                matrix[i][j] + cost,
            );
        }
    }

    matrix[len1][len2]
}

/// Extract section name from error message (e.g., "stop.infinite" -> "stop")
fn extract_section_from_error(error_msg: &str) -> Option<String> {
    // Look for patterns like "stop:", "rules.", "notifications:"
    if let Some(colon_idx) = error_msg.find(':') {
        let before_colon = &error_msg[..colon_idx];
        if let Some(last_word) = before_colon.split_whitespace().last() {
            if let Some(section) = last_word.split('.').next() {
                return Some(section.to_string());
            }
        }
    }
    None
}

/// Format a descriptive error message for YAML parsing failures
fn format_parse_error(error: &serde_yaml::Error, config_path: &Path) -> String {
    let base_error = error.to_string();
    let mut parts = vec![
        format!(
            "Failed to parse configuration file: {}",
            config_path.display()
        ),
        String::new(),
        format!("Error: {}", base_error),
    ];

    // Extract line number if present
    let has_line_number = base_error.contains("at line");

    // Add specific guidance based on error type
    if base_error.contains("unknown field") {
        parts.push(String::new());

        // Try to extract the unknown field and suggest alternatives
        let unknown_field = extract_unknown_field(&base_error);
        let section = extract_section_from_error(&base_error);

        if let (Some(field), Some(sec)) = (unknown_field, section) {
            let suggestions = suggest_similar_fields(&field, &sec);
            if !suggestions.is_empty() {
                parts.push("💡 Did you mean one of these?".to_string());
                for suggestion in &suggestions {
                    parts.push(format!("   • {suggestion}"));
                }
                parts.push(String::new());
            }
        }

        parts.push("Common causes:".to_string());
        parts.push("  • Typo in field name (check spelling and capitalization)".to_string());
        parts.push("  • Using a field that doesn't exist in this section".to_string());
        parts.push("  • Using camelCase vs snake_case incorrectly (use camelCase)".to_string());
        parts.push(String::new());
        parts.push("Valid field names by section:".to_string());
        parts.push("  stop: commands, infinite, infiniteMessage".to_string());
        parts.push("  subagentStop: commands".to_string());
        parts.push(
            "  preToolUse: preventAdditions, preventGeneratedFileEdits, generatedFileMessage, preventRootAdditions, uneditableFiles, preventUpdateGitIgnored, toolUsageValidation"
                .to_string(),
        );
        parts.push(
            "  notifications: enabled, hooks, showErrors, showSuccess, showSystemEvents"
                .to_string(),
        );
        parts.push("  permissionRequest: default, allow, deny".to_string());
        parts.push(
            "  commands (stop): run, message, showStdout, showStderr, maxOutputLines, timeout"
                .to_string(),
        );
        parts.push("  commands (subagentStop): run, message, showStdout, showStderr, maxOutputLines, timeout".to_string());
    } else if base_error.contains("invalid type") {
        parts.push(String::new());
        parts.push("Type mismatch detected. Common causes:".to_string());
        parts.push(
            "  • Using quotes around a boolean value (use true/false without quotes)".to_string(),
        );
        parts.push("  • Using a string where a number is expected (remove quotes)".to_string());
        parts.push("  • Using a single value where an array is expected (wrap in [])".to_string());
        parts.push(String::new());
        parts.push("✅ Examples of correct formatting:".to_string());
        parts.push("   Boolean:  infinite: true             # no quotes".to_string());
        parts.push("   Number:   maxOutputLines: 100        # no quotes".to_string());
        parts.push("   String:   run: \"cargo test\"          # with quotes".to_string());
        parts.push("   Array:    hooks: [\"Stop\"]            # square brackets".to_string());
        parts.push("   Array:    uneditableFiles: []        # empty array".to_string());
    } else if base_error.contains("expected") || base_error.contains("while parsing") {
        parts.push(String::new());
        parts.push("YAML syntax error detected. Common causes:".to_string());
        parts.push(
            "  • Incorrect indentation (YAML requires consistent spaces, not tabs)".to_string(),
        );
        parts.push("  • Missing colon (:) after a field name".to_string());
        parts.push("  • Unmatched quotes or brackets".to_string());
        parts.push("  • Using tabs instead of spaces for indentation".to_string());

        if has_line_number {
            parts.push(String::new());
            parts.push("💡 Check the line number above and the lines around it.".to_string());
        }

        parts.push(String::new());
        parts.push("✅ YAML formatting tips:".to_string());
        parts.push("   • Use 2 spaces for each indentation level".to_string());
        parts.push("   • Always put a space after the colon: 'key: value'".to_string());
        parts.push("   • Use quotes for strings with special characters".to_string());
        parts.push("   • Arrays can be: [item1, item2] or on separate lines with -".to_string());
    } else if base_error.contains("missing field") {
        parts.push(String::new());
        parts.push("A required field is missing from the configuration.".to_string());
        parts.push("Check the default configuration with: conclaude init".to_string());
    }

    parts.push(String::new());
    parts.push("For a valid configuration template, run:".to_string());
    parts.push("  conclaude init".to_string());

    parts.join("\n")
}

/// Parse and validate configuration content from a string
///
/// # Errors
///
/// Returns an error if YAML parsing fails or validation constraints are violated.
pub fn parse_and_validate_config(content: &str, config_path: &Path) -> Result<ConclaudeConfig> {
    let config: ConclaudeConfig = serde_yaml::from_str(content).map_err(|e| {
        let error_msg = format_parse_error(&e, config_path);
        anyhow::anyhow!(error_msg)
    })?;

    validate_config_constraints(&config)?;

    Ok(config)
}

/// Validate configuration values against constraints
fn validate_config_constraints(config: &ConclaudeConfig) -> Result<()> {
    // Validate maxOutputLines range (1-10000)
    for (idx, command) in config.stop.commands.iter().enumerate() {
        if let Some(max_lines) = command.max_output_lines {
            if !(1..=10000).contains(&max_lines) {
                let error_msg = format!(
                    "Range validation failed for stop.commands[{idx}].maxOutputLines\n\n\
                     Error: Value {max_lines} is out of valid range\n\n\
                     ✅ Valid range: 1 to 10000\n\n\
                     Common causes:\n\
                       • Value is too large (maximum is 10000)\n\
                       • Value is too small (minimum is 1)\n\
                       • Using a negative number\n\n\
                     Example valid configurations:\n\
                       maxOutputLines: 100      # default, good for most cases\n\
                       maxOutputLines: 1000     # for verbose output\n\
                       maxOutputLines: 10000    # maximum allowed\n\n\
                     For a valid configuration template, run:\n\
                       conclaude init"
                );
                return Err(anyhow::anyhow!(error_msg));
            }
        }

        // Validate timeout range (1-3600)
        if let Some(timeout) = command.timeout {
            if !(1..=3600).contains(&timeout) {
                let error_msg = format!(
                    "Range validation failed for stop.commands[{idx}].timeout\n\n\
                     Error: Value {timeout} is out of valid range\n\n\
                     ✅ Valid range: 1 to 3600 seconds (1 second to 1 hour)\n\n\
                     Common causes:\n\
                       • Value is too large (maximum is 3600 seconds / 1 hour)\n\
                       • Value is too small (minimum is 1 second)\n\
                       • Using a negative number\n\n\
                     Example valid configurations:\n\
                       timeout: 30       # 30 seconds\n\
                       timeout: 300      # 5 minutes\n\
                       timeout: 3600     # maximum allowed (1 hour)\n\n\
                     For a valid configuration template, run:\n\
                       conclaude init"
                );
                return Err(anyhow::anyhow!(error_msg));
            }
        }
    }

    // Validate permissionRequest.default if specified
    if let Some(permission_request) = &config.permission_request {
        let default_value = permission_request.default.to_lowercase();
        if default_value != "allow" && default_value != "deny" {
            let error_msg = format!(
                "Validation failed for permissionRequest.default\n\n\
                 Error: Invalid value '{}'\n\n\
                 ✅ Valid values: \"allow\" or \"deny\"\n\n\
                 Common causes:\n\
                   • Typo in value (check spelling)\n\
                   • Using a value other than allow or deny\n\n\
                 Example valid configurations:\n\
                   permissionRequest:\n\
                     default: allow    # allow all tools by default\n\
                   \n\
                   permissionRequest:\n\
                     default: deny     # deny all tools by default\n\n\
                 For a valid configuration template, run:\n\
                   conclaude init",
                permission_request.default
            );
            return Err(anyhow::anyhow!(error_msg));
        }
    }

    // Validate subagentStop configuration
    for (pattern, commands) in &config.subagent_stop.commands {
        // Validate pattern is not empty
        if pattern.trim().is_empty() {
            let error_msg = "Validation failed for subagentStop.commands\n\n\
                 Error: Pattern key cannot be empty\n\n\
                 ✅ Valid patterns: \"*\" (all), \"coder\" (exact), \"test*\" (prefix), \"*coder\" (suffix)\n\n\
                 Example valid configurations:\n\
                   subagentStop:\n\
                     commands:\n\
                       \"*\":\n\
                         - run: \"echo all subagents\"\n\
                       \"coder\":\n\
                         - run: \"npm run lint\"\n\n\
                 For a valid configuration template, run:\n\
                   conclaude init"
                .to_string();
            return Err(anyhow::anyhow!(error_msg));
        }

        // Validate maxOutputLines range for each command
        for (idx, command) in commands.iter().enumerate() {
            if let Some(max_lines) = command.max_output_lines {
                if !(1..=10000).contains(&max_lines) {
                    let error_msg = format!(
                        "Range validation failed for subagentStop.commands[\"{pattern}\"][{idx}].maxOutputLines\n\n\
                         Error: Value {max_lines} is out of valid range\n\n\
                         ✅ Valid range: 1 to 10000\n\n\
                         Common causes:\n\
                           • Value is too large (maximum is 10000)\n\
                           • Value is too small (minimum is 1)\n\
                           • Using a negative number\n\n\
                         Example valid configurations:\n\
                           maxOutputLines: 100      # default, good for most cases\n\
                           maxOutputLines: 1000     # for verbose output\n\
                           maxOutputLines: 10000    # maximum allowed\n\n\
                         For a valid configuration template, run:\n\
                           conclaude init"
                    );
                    return Err(anyhow::anyhow!(error_msg));
                }
            }

            // Validate timeout range (1-3600)
            if let Some(timeout) = command.timeout {
                if !(1..=3600).contains(&timeout) {
                    let error_msg = format!(
                        "Range validation failed for subagentStop.commands[\"{pattern}\"][{idx}].timeout\n\n\
                         Error: Value {timeout} is out of valid range\n\n\
                         ✅ Valid range: 1 to 3600 seconds (1 second to 1 hour)\n\n\
                         Common causes:\n\
                           • Value is too large (maximum is 3600 seconds / 1 hour)\n\
                           • Value is too small (minimum is 1 second)\n\
                           • Using a negative number\n\n\
                         Example valid configurations:\n\
                           timeout: 30       # 30 seconds\n\
                           timeout: 300      # 5 minutes\n\
                           timeout: 3600     # maximum allowed (1 hour)\n\n\
                         For a valid configuration template, run:\n\
                           conclaude init"
                    );
                    return Err(anyhow::anyhow!(error_msg));
                }
            }
        }
    }

    Ok(())
}

/// Load YAML configuration using native search strategies
///
/// Search strategy: searches up directory tree from the starting directory,
/// checking for `.conclaude.yaml` or `.conclaude.yml` in each parent directory.
/// The search stops when either:
/// - A config file is found, OR
/// - The filesystem root is reached, OR
/// - 12 directory levels have been searched
///
/// # Arguments
///
/// * `start_dir` - Optional starting directory for config search. If None, uses current directory.
///
/// # Errors
///
/// Returns an error if no configuration file is found, file reading fails, or YAML parsing fails.
pub async fn load_conclaude_config(start_dir: Option<&Path>) -> Result<(ConclaudeConfig, PathBuf)> {
    let search_paths = get_config_search_paths(start_dir)?;

    for path in &search_paths {
        if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;

            let config = parse_and_validate_config(&content, path)?;

            return Ok((config, path.clone()));
        }
    }

    // If no config file is found, show search locations
    let search_locations: Vec<String> = search_paths
        .iter()
        .map(|p| format!("  • {}", p.display()))
        .collect();

    let error_message = format!(
        "Configuration file not found.\n\nSearched the following locations:\n{}\n\nCreate a .conclaude.yaml or .conclaude.yml file with stop and preToolUse sections.\nRun 'conclaude init' to generate a template configuration.",
        search_locations.join("\n")
    );

    Err(anyhow::anyhow!(error_message))
}

fn get_config_search_paths(start_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut current_dir = match start_dir {
        Some(dir) => dir.to_path_buf(),
        None => std::env::current_dir()?,
    };
    let mut levels_searched = 0;
    const MAX_SEARCH_LEVELS: u32 = 12;

    loop {
        // Add .conclaude.yaml and .conclaude.yml to search paths
        paths.push(current_dir.join(".conclaude.yaml"));
        paths.push(current_dir.join(".conclaude.yml"));

        // Move to parent directory first, then increment level count
        match current_dir.parent() {
            Some(parent) => {
                current_dir = parent.to_path_buf();
                levels_searched += 1;

                // Check if we've reached the maximum search level limit
                if levels_searched >= MAX_SEARCH_LEVELS {
                    break;
                }
            }
            None => break, // Reached filesystem root
        }
    }

    Ok(paths)
}

/// Extracts individual commands from a bash script string
///
/// # Errors
///
/// Returns an error if the bash command execution fails or UTF-8 parsing fails.
pub fn extract_bash_commands(bash_script: &str) -> Result<Vec<String>> {
    let analyzer_script = format!(
        r#"#!/bin/bash
# This script outputs plain text lines, NOT JSON

# Process each line of the input script
while IFS= read -r line; do
  # Skip empty lines and comments
  if [[ -z "${{line// }}" ]] || [[ "$line" =~ ^[[:space:]]*# ]]; then
    continue
  fi
  
  # Output in a simple delimited format (NOT JSON)
  echo "CMD:$line"
done << 'EOF'
{bash_script}
EOF"#
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(&analyzer_script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute bash command analyzer")?;

    let mut commands = Vec::new();

    if !output.stdout.is_empty() {
        let stdout = String::from_utf8(output.stdout)
            .context("Failed to parse bash analyzer stdout as UTF-8")?;

        for line in stdout.lines() {
            if let Some(command) = line.strip_prefix("CMD:") {
                if !command.is_empty() {
                    commands.push(command.to_string());
                }
            }
        }
    }

    // Check for errors
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Bash reported errors: {stderr}");
    }

    Ok(commands)
}

/// Generate a default configuration file content
/// The configuration is embedded at compile time from default-config.yaml
#[must_use]
pub fn generate_default_config() -> String {
    include_str!("default-config.yaml").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bash_commands_single() {
        let script = "echo hello";
        let commands = extract_bash_commands(script).unwrap();
        assert_eq!(commands, vec!["echo hello"]);
    }

    #[test]
    fn test_extract_bash_commands_multiple() {
        let script = "echo hello\nnpm install\nnpm test";
        let commands = extract_bash_commands(script).unwrap();
        assert_eq!(commands, vec!["echo hello", "npm install", "npm test"]);
    }

    #[test]
    fn test_extract_bash_commands_ignores_comments() {
        let script = "# This is a comment\necho hello\n# Another comment\nnpm test";
        let commands = extract_bash_commands(script).unwrap();
        assert_eq!(commands, vec!["echo hello", "npm test"]);
    }

    #[test]
    fn test_extract_bash_commands_ignores_empty_lines() {
        let script = "echo hello\n\nnpm test\n";
        let commands = extract_bash_commands(script).unwrap();
        assert_eq!(commands, vec!["echo hello", "npm test"]);
    }

    #[test]
    fn test_extract_bash_commands_complex() {
        let script = r#"nix develop -c "lint"
bun x tsc --noEmit
cd /tmp && echo "test""#;
        let commands = extract_bash_commands(script).unwrap();
        assert_eq!(
            commands,
            vec![
                r#"nix develop -c "lint""#,
                "bun x tsc --noEmit",
                r#"cd /tmp && echo "test""#
            ]
        );
    }

    #[test]
    fn test_field_list_generation() {
        // Verify that the generated field_names() methods return the correct field names
        assert_eq!(
            StopConfig::field_names(),
            vec!["commands", "infinite", "infiniteMessage"]
        );

        assert_eq!(
            PreToolUseConfig::field_names(),
            vec![
                "preventAdditions",
                "preventGeneratedFileEdits",
                "generatedFileMessage",
                "preventRootAdditions",
                "uneditableFiles",
                "preventUpdateGitIgnored",
                "toolUsageValidation"
            ]
        );

        assert_eq!(
            NotificationsConfig::field_names(),
            vec![
                "enabled",
                "hooks",
                "showErrors",
                "showSuccess",
                "showSystemEvents"
            ]
        );

        assert_eq!(
            StopCommand::field_names(),
            vec![
                "run",
                "message",
                "showStdout",
                "showStderr",
                "maxOutputLines",
                "timeout"
            ]
        );
    }

    #[test]
    fn test_suggest_similar_fields_common_typo() {
        // Test common typo: "showStdOut" should suggest "showStdout"
        let suggestions = suggest_similar_fields("showStdOut", "commands");
        assert!(
            !suggestions.is_empty(),
            "Should suggest fields for common typo"
        );
        assert_eq!(
            suggestions[0], "showStdout",
            "First suggestion should be 'showStdout'"
        );
    }

    #[test]
    fn test_suggest_similar_fields_case_insensitive() {
        // Test case-insensitive matching: "INFINITE" should suggest "infinite"
        let suggestions = suggest_similar_fields("INFINITE", "stop");
        assert!(
            !suggestions.is_empty(),
            "Should suggest fields ignoring case"
        );
        assert!(
            suggestions.contains(&"infinite".to_string()),
            "Should suggest 'infinite' for 'INFINITE'"
        );
    }

    #[test]
    fn test_suggest_similar_fields_distance_threshold() {
        // Test that only suggestions within distance 3 are returned
        // "infinit" (distance 1) should be suggested
        let suggestions = suggest_similar_fields("infinit", "stop");
        assert!(
            suggestions.contains(&"infinite".to_string()),
            "Should suggest 'infinite' for 'infinit' (distance 1)"
        );

        // "infinte" (distance 1, missing 'i') should be suggested
        let suggestions = suggest_similar_fields("infinte", "stop");
        assert!(
            suggestions.contains(&"infinite".to_string()),
            "Should suggest 'infinite' for 'infinte' (distance 1)"
        );

        // "wxyz" has distance > 3 from all stop fields, should not suggest anything
        let suggestions = suggest_similar_fields("wxyz", "stop");
        assert!(
            suggestions.is_empty(),
            "Should not suggest anything for 'wxyz' (distance > 3 from all fields)"
        );
    }

    #[test]
    fn test_suggest_similar_fields_no_close_matches() {
        // Test that empty results are returned when no close matches exist
        let suggestions = suggest_similar_fields("completelywrongfield", "stop");
        assert!(
            suggestions.is_empty(),
            "Should return empty for field with no close matches"
        );

        let suggestions = suggest_similar_fields("abcdefgh", "rules");
        assert!(
            suggestions.is_empty(),
            "Should return empty when distance exceeds threshold"
        );
    }

    #[test]
    fn test_suggest_similar_fields_sorted_by_distance() {
        // Test that suggestions are sorted by distance (closest first)
        // "messag" (distance 1 from "message") should come before anything with higher distance
        let suggestions = suggest_similar_fields("messag", "commands");
        if !suggestions.is_empty() {
            assert_eq!(
                suggestions[0], "message",
                "Closest match should be first in suggestions"
            );
        }
    }

    #[test]
    fn test_suggest_similar_fields_max_three_suggestions() {
        // Test that at most 3 suggestions are returned
        let suggestions = suggest_similar_fields("sho", "commands");
        assert!(
            suggestions.len() <= 3,
            "Should return at most 3 suggestions, got {}",
            suggestions.len()
        );
    }

    #[test]
    fn test_suggest_similar_fields_invalid_section() {
        // Test that empty results are returned for invalid section
        let suggestions = suggest_similar_fields("infinite", "invalid_section");
        assert!(
            suggestions.is_empty(),
            "Should return empty for invalid section"
        );
    }

    #[test]
    fn test_suggest_similar_fields_notifications_section() {
        // Test suggestions for notifications section
        let suggestions = suggest_similar_fields("enable", "notifications");
        assert!(
            suggestions.contains(&"enabled".to_string()),
            "Should suggest 'enabled' for 'enable' in notifications section"
        );
    }

    #[test]
    fn test_suggest_similar_fields_pretooluse_section() {
        // Test suggestions for preToolUse section with camelCase field
        let suggestions = suggest_similar_fields("preventRootAddition", "preToolUse");
        assert!(
            suggestions.contains(&"preventRootAdditions".to_string()),
            "Should suggest 'preventRootAdditions' for 'preventRootAddition'"
        );
    }

    #[test]
    fn test_config_without_rules_section_works() {
        // Test that configuration without rules section works normally
        let valid_config = r#"
preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  preventGeneratedFileEdits: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;

        let result = parse_and_validate_config(valid_config, Path::new("test.yaml"));
        assert!(
            result.is_ok(),
            "Should accept config without rules section: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_permission_request_valid_config() {
        // Test valid permissionRequest configuration
        let config_yaml = r#"
permissionRequest:
  default: allow
  allow:
    - Read
    - Write
  deny:
    - Bash

stop:
  commands: []
  infinite: false

preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  preventGeneratedFileEdits: true
  toolUsageValidation: []

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;

        let result = parse_and_validate_config(config_yaml, Path::new("test.yaml"));
        assert!(
            result.is_ok(),
            "Valid permissionRequest config should parse: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert!(
            config.permission_request.is_some(),
            "permission_request should be populated"
        );
        let pr = config.permission_request.unwrap();
        assert_eq!(pr.default, "allow");
        assert_eq!(pr.allow.as_ref().unwrap().len(), 2);
        assert_eq!(pr.deny.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_permission_request_invalid_default() {
        // Test that invalid default value is rejected
        let config_yaml = r#"
permissionRequest:
  default: invalid_value

stop:
  commands: []
  infinite: false

preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  preventGeneratedFileEdits: true
  toolUsageValidation: []

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;

        let result = parse_and_validate_config(config_yaml, Path::new("test.yaml"));
        assert!(
            result.is_err(),
            "Invalid default value should fail validation"
        );
        let error = result.err().unwrap().to_string();
        assert!(
            error.contains("allow") && error.contains("deny"),
            "Error message should mention valid values"
        );
    }

    #[test]
    fn test_permission_request_optional() {
        // Test that permissionRequest is optional
        let config_yaml = r#"
stop:
  commands: []
  infinite: false

preToolUse:
  preventRootAdditions: true
  uneditableFiles: []
  preventAdditions: []
  preventGeneratedFileEdits: true
  toolUsageValidation: []

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;

        let result = parse_and_validate_config(config_yaml, Path::new("test.yaml"));
        assert!(
            result.is_ok(),
            "Config without permissionRequest should parse: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert!(
            config.permission_request.is_none(),
            "permission_request should be None when not specified"
        );
    }

    #[test]
    fn test_permission_request_field_list() {
        // Test that PermissionRequestConfig field names are correct
        assert_eq!(
            PermissionRequestConfig::field_names(),
            vec!["default", "allow", "deny"]
        );
    }

    #[test]
    fn test_uneditable_file_rule_simple_string_format() {
        // Test that simple string patterns deserialize correctly
        let yaml = r#"
preToolUse:
  uneditableFiles:
    - "*.lock"
    - ".env"
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.pre_tool_use.uneditable_files.len(), 2);

        // Verify patterns extracted correctly
        assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
        assert_eq!(config.pre_tool_use.uneditable_files[1].pattern(), ".env");

        // Verify no custom messages
        assert!(config.pre_tool_use.uneditable_files[0].message().is_none());
        assert!(config.pre_tool_use.uneditable_files[1].message().is_none());
    }

    #[test]
    fn test_uneditable_file_rule_detailed_object_format() {
        // Test that detailed objects with pattern and message deserialize correctly
        let yaml = r#"
preToolUse:
  uneditableFiles:
    - pattern: "*.lock"
      message: "Lock files are auto-generated. Run 'npm install' to update."
    - pattern: ".env"
      message: "Environment files contain secrets."
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.pre_tool_use.uneditable_files.len(), 2);

        // Verify patterns extracted correctly
        assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
        assert_eq!(config.pre_tool_use.uneditable_files[1].pattern(), ".env");

        // Verify custom messages
        assert_eq!(
            config.pre_tool_use.uneditable_files[0].message(),
            Some("Lock files are auto-generated. Run 'npm install' to update.")
        );
        assert_eq!(
            config.pre_tool_use.uneditable_files[1].message(),
            Some("Environment files contain secrets.")
        );
    }

    #[test]
    fn test_uneditable_file_rule_mixed_format() {
        // Test that arrays can mix both simple strings and detailed objects
        let yaml = r#"
preToolUse:
  uneditableFiles:
    - "*.lock"
    - pattern: ".env"
      message: "Secrets must not be committed."
    - "package.json"
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.pre_tool_use.uneditable_files.len(), 3);

        // First is simple format
        assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
        assert!(config.pre_tool_use.uneditable_files[0].message().is_none());

        // Second is detailed format with message
        assert_eq!(config.pre_tool_use.uneditable_files[1].pattern(), ".env");
        assert_eq!(
            config.pre_tool_use.uneditable_files[1].message(),
            Some("Secrets must not be committed.")
        );

        // Third is simple format
        assert_eq!(
            config.pre_tool_use.uneditable_files[2].pattern(),
            "package.json"
        );
        assert!(config.pre_tool_use.uneditable_files[2].message().is_none());
    }

    #[test]
    fn test_uneditable_file_rule_detailed_without_message() {
        // Test that detailed format without message field works (message is optional)
        let yaml = r#"
preToolUse:
  uneditableFiles:
    - pattern: "*.lock"
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let config: ConclaudeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.pre_tool_use.uneditable_files.len(), 1);
        assert_eq!(config.pre_tool_use.uneditable_files[0].pattern(), "*.lock");
        assert!(config.pre_tool_use.uneditable_files[0].message().is_none());
    }

    #[test]
    fn test_uneditable_file_rule_backward_compatibility() {
        // Test that existing configs with simple string format still work
        let yaml = r#"
preToolUse:
  preventRootAdditions: true
  uneditableFiles:
    - "*.lock"
    - ".env"
    - "package-lock.json"
  preventAdditions: []
  preventGeneratedFileEdits: true
  toolUsageValidation: []

stop:
  commands: []
  infinite: false

notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;

        let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
        assert!(
            result.is_ok(),
            "Backward compatible config should parse: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert_eq!(config.pre_tool_use.uneditable_files.len(), 3);

        // Verify all patterns are extracted correctly
        let patterns: Vec<&str> = config
            .pre_tool_use
            .uneditable_files
            .iter()
            .map(|r| r.pattern())
            .collect();
        assert_eq!(patterns, vec!["*.lock", ".env", "package-lock.json"]);
    }

    #[test]
    fn test_uneditable_file_rule_pattern_extraction() {
        // Test the pattern() method for both variants
        let simple = UnEditableFileRule::Simple("*.txt".to_string());
        assert_eq!(simple.pattern(), "*.txt");

        let detailed = UnEditableFileRule::Detailed {
            pattern: "*.md".to_string(),
            message: Some("Custom message".to_string()),
        };
        assert_eq!(detailed.pattern(), "*.md");
    }

    #[test]
    fn test_uneditable_file_rule_message_extraction() {
        // Test the message() method for all cases
        let simple = UnEditableFileRule::Simple("*.txt".to_string());
        assert!(simple.message().is_none());

        let detailed_with_msg = UnEditableFileRule::Detailed {
            pattern: "*.md".to_string(),
            message: Some("Custom message".to_string()),
        };
        assert_eq!(detailed_with_msg.message(), Some("Custom message"));

        let detailed_without_msg = UnEditableFileRule::Detailed {
            pattern: "*.md".to_string(),
            message: None,
        };
        assert!(detailed_without_msg.message().is_none());
    }

    #[test]
    fn test_stop_command_timeout_parsing() {
        let yaml = r#"
stop:
  commands:
    - run: "sleep 10"
      timeout: 30
    - run: "echo hello"
preToolUse:
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
        assert!(
            result.is_ok(),
            "Config with timeout should parse: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert_eq!(config.stop.commands.len(), 2);
        assert_eq!(config.stop.commands[0].timeout, Some(30));
        assert_eq!(config.stop.commands[1].timeout, None);
    }

    #[test]
    fn test_stop_command_timeout_invalid_too_large() {
        let yaml = r#"
stop:
  commands:
    - run: "echo test"
      timeout: 3601
preToolUse:
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
        assert!(
            result.is_err(),
            "Config with timeout > 3600 should fail validation"
        );
        let error = result.err().unwrap().to_string();
        assert!(
            error.contains("timeout") || error.contains("3600"),
            "Error should mention timeout issue: {}",
            error
        );
    }

    #[test]
    fn test_stop_command_timeout_invalid_zero() {
        let yaml = r#"
stop:
  commands:
    - run: "echo test"
      timeout: 0
preToolUse:
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
        assert!(
            result.is_err(),
            "Config with timeout = 0 should fail validation"
        );
    }

    #[test]
    fn test_subagent_stop_command_timeout_parsing() {
        let yaml = r#"
subagentStop:
  commands:
    "*":
      - run: "npm run lint"
        timeout: 60
stop:
  commands: []
preToolUse:
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
        assert!(
            result.is_ok(),
            "Config with subagent timeout should parse: {:?}",
            result.err()
        );

        let config = result.unwrap();
        let cmds = config.subagent_stop.commands.get("*").unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].timeout, Some(60));
    }

    #[test]
    fn test_timeout_backward_compatibility() {
        let yaml = r#"
stop:
  commands:
    - run: "echo hello"
      message: "Testing"
      showStdout: true
preToolUse:
  preventAdditions: []
  preventGeneratedFileEdits: true
  preventRootAdditions: true
  uneditableFiles: []
  toolUsageValidation: []
notifications:
  enabled: false
  hooks: []
  showErrors: false
  showSuccess: false
  showSystemEvents: true
"#;
        let result = parse_and_validate_config(yaml, Path::new("test.yaml"));
        assert!(
            result.is_ok(),
            "Config without timeout should still parse: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert_eq!(config.stop.commands[0].timeout, None);
    }

    #[test]
    fn test_stop_command_field_list_includes_timeout() {
        let fields = StopCommand::field_names();
        assert!(
            fields.contains(&"timeout"),
            "StopCommand field_names should include 'timeout'"
        );
    }
}

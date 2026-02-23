use crate::config::{
    extract_bash_commands, load_conclaude_config, ConclaudeConfig, ConfigChangeConfig,
    SkillStartConfig, SlashCommandConfig, SubagentStopConfig, TaskCompletedConfig,
    TeammateIdleConfig, UserPromptSubmitCommand,
};
use crate::gitignore::{find_git_root, is_path_git_ignored};
use crate::types::{
    validate_base_payload, validate_permission_request_payload, validate_subagent_start_payload,
    validate_subagent_stop_payload, validate_task_completed_payload, validate_teammate_idle_payload,
    validate_worktree_create_payload, validate_worktree_remove_payload, ConfigChangePayload,
    ConfigChangeSource, HookResult, NotificationPayload, PermissionRequestPayload,
    PostToolUseFailurePayload, PostToolUsePayload, PreCompactPayload, PreToolUsePayload,
    SessionEndPayload, SessionStartPayload, StopPayload, SubagentStartPayload,
    SubagentStopPayload, TaskCompletedPayload, TeammateIdlePayload, UserPromptSubmitPayload,
    WorktreeCreatePayload, WorktreeRemovePayload,
};
use anyhow::{Context, Result};
use glob::Pattern;
use notify_rust::{Notification, Urgency};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration};

/// Environment variable name for passing agent context to hook handlers
const AGENT_ENV_VAR: &str = "CONCLAUDE_AGENT";
/// Get the path to the agent session file for a given session.
#[allow(dead_code)]
pub fn get_agent_session_file_path(session_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("conclaude-agent-{}.json", session_id))
}

/// Write agent info to session file during SubagentStart.
///
/// # Errors
///
/// Returns an error if the session file cannot be written.
#[allow(dead_code)]
pub fn write_agent_session_file(session_id: &str, subagent_type: &str) -> std::io::Result<()> {
    let path = get_agent_session_file_path(session_id);
    let content = serde_json::json!({
        "subagent_type": subagent_type
    });
    fs::write(&path, content.to_string())
}

/// Read agent info from session file during PreToolUse.
/// Returns "main" if no session file exists (we're in the orchestrator session).
#[allow(dead_code)]
#[must_use]
pub fn read_agent_from_session_file(session_id: &str) -> String {
    let path = get_agent_session_file_path(session_id);
    match fs::read_to_string(&path) {
        Ok(content) => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                json.get("subagent_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "main".to_string())
            } else {
                "main".to_string()
            }
        }
        Err(_) => "main".to_string(), // No file = main session
    }
}

/// Represents a stop command with its configuration
pub(crate) struct StopCommandConfig {
    pub(crate) command: String,
    pub(crate) message: Option<String>,
    pub(crate) show_stdout: bool,
    pub(crate) show_stderr: bool,
    pub(crate) max_output_lines: Option<u32>,
    pub(crate) timeout: Option<u64>,
    pub(crate) show_command: bool,
    pub(crate) notify_per_command: bool,
}

/// Represents a subagent stop command with its configuration
pub(crate) struct SubagentStopCommandConfig {
    pub(crate) command: String,
    pub(crate) message: Option<String>,
    pub(crate) show_stdout: bool,
    pub(crate) show_stderr: bool,
    pub(crate) max_output_lines: Option<u32>,
    pub(crate) timeout: Option<u64>,
    pub(crate) show_command: bool,
    pub(crate) notify_per_command: bool,
}

/// Represents a user prompt submit command with its configuration
pub(crate) struct UserPromptSubmitCommandConfig {
    pub(crate) command: String,
    pub(crate) show_stdout: bool,
    pub(crate) show_stderr: bool,
    pub(crate) max_output_lines: Option<u32>,
    pub(crate) timeout: Option<u64>,
    pub(crate) show_command: bool,
    pub(crate) notify_per_command: bool,
}

/// Represents a slash command entry with its configuration
pub(crate) struct SlashCommandEntryConfig {
    pub(crate) command: String,
    pub(crate) show_stdout: bool,
    pub(crate) show_stderr: bool,
    pub(crate) max_output_lines: Option<u32>,
    pub(crate) timeout: Option<u64>,
    pub(crate) show_command: bool,
    pub(crate) notify_per_command: bool,
}

/// Represents a skill start command with its configuration
pub(crate) struct SkillStartCommandConfig {
    pub(crate) command: String,
    pub(crate) show_stdout: bool,
    pub(crate) show_stderr: bool,
    pub(crate) max_output_lines: Option<u32>,
    pub(crate) timeout: Option<u64>,
    pub(crate) show_command: bool,
    pub(crate) notify_per_command: bool,
}

/// Result of detecting a slash command in prompt text
#[derive(Debug, Clone)]
pub(crate) struct SlashCommandDetection {
    pub(crate) command: String,
    pub(crate) args: String,
}

/// Detect slash command from prompt text
///
/// Parses the prompt for patterns matching `/^\/(\w+)(?:\s+(.*))?$/m`
/// at the start of the prompt or after newlines.
///
/// # Arguments
///
/// * `prompt` - The user prompt text to analyze
///
/// # Returns
///
/// `Some(SlashCommandDetection)` if a slash command is found, `None` otherwise
pub(crate) fn detect_slash_command(prompt: &str) -> Option<SlashCommandDetection> {
    // Split prompt into lines and check each line for slash command
    for line in prompt.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('/') {
            // Extract command name (alphanumeric and underscore only)
            let command_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                .unwrap_or(rest.len());

            if command_end > 0 {
                let command = rest[..command_end].to_string();
                let args = rest[command_end..].trim().to_string();

                return Some(SlashCommandDetection { command, args });
            }
        }
    }

    None
}

/// Cached configuration instance to avoid repeated loads
static CACHED_CONFIG: OnceLock<(ConclaudeConfig, std::path::PathBuf)> = OnceLock::new();

/// Determine if a hook is a system event hook
///
/// System event hooks are hooks that track session lifecycle and user interactions,
/// as opposed to tool execution or validation hooks.
///
/// # Arguments
///
/// * `hook_name` - The name of the hook to check
///
/// # Returns
///
/// `true` if the hook is a system event hook, `false` otherwise
#[must_use]
pub(crate) fn is_system_event_hook(hook_name: &str) -> bool {
    matches!(
        hook_name,
        "SessionStart"
            | "SessionEnd"
            | "UserPromptSubmit"
            | "SubagentStart"
            | "SubagentStop"
            | "PreCompact"
            | "TeammateIdle"
            | "TaskCompleted"
            | "ConfigChange"
            | "WorktreeCreate"
            | "WorktreeRemove"
    )
}

/// Load configuration with caching to avoid repeated file system operations
///
/// # Errors
///
/// Returns an error if the configuration file cannot be loaded or parsed.
async fn get_config() -> Result<&'static (ConclaudeConfig, std::path::PathBuf)> {
    if let Some(config) = CACHED_CONFIG.get() {
        Ok(config)
    } else {
        let config = load_conclaude_config(None).await?;
        Ok(CACHED_CONFIG.get_or_init(|| config))
    }
}

/// Extract the directory containing the config file
///
/// Normalizes empty parent paths (when config is in CWD) to "." for consistent
/// directory reference across the codebase.
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
///
/// # Returns
///
/// The directory containing the config file, or "." if the parent is empty
#[must_use]
fn get_config_dir(config_path: &Path) -> &Path {
    config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

/// Send a system notification for hook execution
///
/// This function sends a system notification when a hook is executed.
/// It gracefully handles errors and logs failures without blocking hook execution.
///
/// # Arguments
///
/// * `hook_name` - The name of the hook being executed
/// * `status` - The execution status ("success" or "failure")
/// * `context` - Optional additional context about the execution
fn send_notification(hook_name: &str, status: &str, context: Option<&str>) {
    // Get configuration to check if notifications are enabled for this hook
    let config_future = get_config();

    // Use tokio::task::block_in_place to safely block in async context
    let config_result =
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(config_future));

    let (config, _) = match config_result {
        Ok(config) => config,
        Err(e) => {
            // Silently continue if config can't be loaded - notifications are not critical
            eprintln!("Failed to load config for notification: {e}");
            return;
        }
    };

    // Check if notifications are enabled for this hook
    if !config.notifications.is_enabled_for(hook_name) {
        return;
    }

    // Check notification flags based on hook type and status
    let notifications_config = &config.notifications;

    // Determine if this hook should show based on the appropriate flag
    let should_show = match status {
        "failure" => notifications_config.show_errors,
        "success" => notifications_config.show_success,
        _ => {
            // For other statuses, determine if this is a system event hook
            is_system_event_hook(hook_name) && notifications_config.show_system_events
        }
    };

    // Short-circuit if the appropriate flag is false
    if !should_show {
        return;
    }

    // Format notification title and body
    let title = format!("Conclaude - {}", hook_name);
    let body = match context {
        Some(ctx) => format!("{}: {}", status, ctx),
        None => match status {
            "success" => "All checks passed".to_string(),
            "failure" => "Command failed".to_string(),
            _ => format!("Hook completed with status: {}", status),
        },
    };

    // Set urgency based on status (Linux only)
    #[cfg(target_os = "linux")]
    let urgency = if status == "failure" {
        Urgency::Critical
    } else {
        Urgency::Normal
    };

    // Send notification with error handling
    #[cfg(target_os = "linux")]
    let result = Notification::new()
        .summary(&title)
        .body(&body)
        .urgency(urgency)
        .show();

    #[cfg(not(target_os = "linux"))]
    let result = Notification::new().summary(&title).body(&body).show();

    match result {
        Ok(_) => {
            // Notification sent successfully
        }
        Err(e) => {
            // Log the error but don't fail the hook
            eprintln!("Failed to send system notification for hook '{hook_name}': {e}");
        }
    }
}

/// Reads and deserializes the hook payload from stdin.
///
/// # Errors
///
/// Returns an error if reading from stdin fails or if the JSON payload cannot be parsed.
pub fn read_payload_from_stdin<T>() -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    let payload: T =
        serde_json::from_str(&buffer).context("Failed to parse JSON payload from stdin")?;

    Ok(payload)
}

/// Wrapper function that standardizes hook result processing and process exit codes.
///
/// # Errors
///
/// Returns an error if the hook handler fails to execute.
///
/// # Panics
///
/// This function does not panic - the `unwrap()` call is guarded by `is_some()` check.
pub async fn handle_hook_result<F, Fut>(handler: F) -> Result<()>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<HookResult>>,
{
    match handler().await {
        Ok(result) => {
            // Serialize the result to JSON and output to stdout
            let json = serde_json::to_string(&result)
                .context("Failed to serialize hook result to JSON")?;
            println!("{}", json);

            // If blocked, also print the message to stderr for visibility
            if result.blocked.unwrap_or(false) {
                if let Some(ref message) = result.message {
                    eprintln!("{}", message);
                }
                std::process::exit(2);
            }
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

/// Handles `PreToolUse` hook events fired before Claude executes any tool.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
pub async fn handle_pre_tool_use() -> Result<HookResult> {
    let payload: PreToolUsePayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    if payload.tool_name.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: tool_name"));
    }

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing PreToolUse hook: session_id={}, tool_name={}",
        payload.base.session_id, payload.tool_name
    );

    // Check tool usage validation rules
    if let Some(result) = check_tool_usage_rules(&payload).await? {
        send_notification(
            "PreToolUse",
            "failure",
            Some(&format!(
                "Tool '{}' blocked by validation rules",
                payload.tool_name
            )),
        );
        return Ok(result);
    }

    let file_modifying_tools = ["Write", "Edit", "MultiEdit", "NotebookEdit"];

    if file_modifying_tools.contains(&payload.tool_name.as_str()) {
        // Check if file is git-ignored and should not be modified
        if let Some(result) = check_git_ignored_file(&payload).await? {
            send_notification(
                "PreToolUse",
                "failure",
                Some(&format!(
                    "Git-ignored file protection blocked tool '{}'",
                    payload.tool_name
                )),
            );
            return Ok(result);
        }

        if let Some(result) = check_file_validation_rules(&payload).await? {
            send_notification(
                "PreToolUse",
                "failure",
                Some(&format!(
                    "File validation failed for tool '{}'",
                    payload.tool_name
                )),
            );
            return Ok(result);
        }
    }

    // Send notification for successful pre-tool-use validation
    send_notification(
        "PreToolUse",
        "success",
        Some(&format!("Tool '{}' approved", payload.tool_name)),
    );
    Ok(HookResult::success())
}

/// Handles `PermissionRequest` hook events fired when Claude requests permission to execute a tool.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
pub async fn handle_permission_request() -> Result<HookResult> {
    let payload: PermissionRequestPayload = read_payload_from_stdin()?;

    validate_permission_request_payload(&payload).map_err(|e| anyhow::anyhow!(e))?;

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing PermissionRequest hook: session_id={}, tool_name={}",
        payload.base.session_id, payload.tool_name
    );

    let (config, _config_path) = get_config().await?;

    // If no permission_request config section exists, default to permissive mode (allow)
    let Some(permission_config) = &config.permission_request else {
        send_notification(
            "PermissionRequest",
            "success",
            Some(&format!("Tool '{}' allowed (no config)", payload.tool_name)),
        );
        return Ok(HookResult::success());
    };

    // Check deny patterns first (deny takes precedence)
    if let Some(deny_patterns) = &permission_config.deny {
        for pattern_str in deny_patterns {
            let pattern = Pattern::new(pattern_str)
                .with_context(|| format!("Invalid glob pattern in deny list: {}", pattern_str))?;
            if pattern.matches(&payload.tool_name) {
                let message = format!(
                    "Tool '{}' blocked by permissionRequest.deny pattern: {}",
                    payload.tool_name, pattern_str
                );
                eprintln!(
                    "PermissionRequest blocked by deny pattern: tool_name={}, pattern={}",
                    payload.tool_name, pattern_str
                );
                send_notification(
                    "PermissionRequest",
                    "failure",
                    Some(&format!("Tool '{}' denied", payload.tool_name)),
                );
                return Ok(HookResult::blocked(message));
            }
        }
    }

    // Check allow patterns second
    if let Some(allow_patterns) = &permission_config.allow {
        for pattern_str in allow_patterns {
            let pattern = Pattern::new(pattern_str)
                .with_context(|| format!("Invalid glob pattern in allow list: {}", pattern_str))?;
            if pattern.matches(&payload.tool_name) {
                send_notification(
                    "PermissionRequest",
                    "success",
                    Some(&format!("Tool '{}' allowed", payload.tool_name)),
                );
                return Ok(HookResult::success());
            }
        }
    }

    // No patterns matched - use default setting
    let default_action = permission_config.default.to_lowercase();
    if default_action == "allow" {
        send_notification(
            "PermissionRequest",
            "success",
            Some(&format!("Tool '{}' allowed by default", payload.tool_name)),
        );
        Ok(HookResult::success())
    } else {
        // default is "deny"
        let message = format!(
            "Tool '{}' blocked by permissionRequest.default setting",
            payload.tool_name
        );
        eprintln!(
            "PermissionRequest blocked by default: tool_name={}",
            payload.tool_name
        );
        send_notification(
            "PermissionRequest",
            "failure",
            Some(&format!("Tool '{}' denied by default", payload.tool_name)),
        );
        Ok(HookResult::blocked(message))
    }
}

/// Check file validation rules for file-modifying tools
///
/// # Errors
///
/// Returns an error if configuration loading fails, directory access fails, or glob pattern processing fails.
async fn check_file_validation_rules(payload: &PreToolUsePayload) -> Result<Option<HookResult>> {
    let (config, config_path) = get_config().await?;

    // Extract file path from tool input
    let file_path = extract_file_path(&payload.tool_input);
    let Some(file_path) = file_path else {
        return Ok(None);
    };

    let cwd = std::env::current_dir().context("Failed to get current working directory")?;
    let resolved_path = cwd.join(&file_path);
    let relative_path = resolved_path
        .strip_prefix(&cwd)
        .unwrap_or(resolved_path.as_path())
        .to_string_lossy()
        .to_string();

    // Check preventRootAdditions rule - only applies to Write tool for NEW files
    // File existence check allows modifications to existing root files (e.g., package.json)
    // but prevents creation of new files at root
    if config.pre_tool_use.prevent_root_additions
        && payload.tool_name == "Write"
        && is_root_addition(&file_path, &relative_path, config_path)
        && !resolved_path.exists()
    {
        // Use custom message if configured, otherwise use default
        let error_message = if let Some(custom_msg) =
            &config.pre_tool_use.prevent_root_additions_message
        {
            custom_msg
                .replace("{file_path}", &file_path)
                .replace("{tool}", &payload.tool_name)
        } else {
            format!(
                "Blocked {} operation: preToolUse.preventRootAdditions setting prevents creating files at repository root. File: {}",
                payload.tool_name, file_path
            )
        };

        eprintln!(
            "PreToolUse blocked by preToolUse.preventRootAdditions setting: tool_name={}, file_path={}",
            payload.tool_name, file_path
        );

        return Ok(Some(HookResult::blocked(error_message)));
    }

    // Detect current agent context from environment variable (set by CLI --agent flag)
    let current_agent = std::env::var(AGENT_ENV_VAR).unwrap_or_else(|_| "main".to_string());

    // Check uneditableFiles rule
    for rule in &config.pre_tool_use.uneditable_files {
        // Check agent match first - skip rule if it doesn't apply to current agent
        let agent_pattern = rule.agent().unwrap_or("*");
        if !matches_agent_pattern(&current_agent, agent_pattern) {
            continue; // Rule doesn't apply to this agent
        }

        let pattern = rule.pattern();
        if matches_uneditable_pattern(
            &file_path,
            &relative_path,
            &resolved_path.to_string_lossy(),
            pattern,
        )? {
            // Include agent context in error message when agent-specific rule triggered
            let agent_suffix = if agent_pattern != "*" {
                format!(" (agent: {})", current_agent)
            } else {
                String::new()
            };

            let error_message = if let Some(custom_msg) = rule.message() {
                format!("{}{}", custom_msg, agent_suffix)
            } else {
                format!(
                    "Blocked {} operation: file matches preToolUse.uneditableFiles pattern '{}'{} File: {}",
                    payload.tool_name, pattern, agent_suffix, file_path
                )
            };

            eprintln!(
                "PreToolUse blocked by preToolUse.uneditableFiles pattern: tool_name={}, file_path={}, pattern={}, agent={}",
                payload.tool_name, file_path, pattern, current_agent
            );

            return Ok(Some(HookResult::blocked(error_message)));
        }
    }

    // Check preventAdditions rule - only applies to Write tool creating NEW files
    // Existing files can be overwritten (preventAdditions only blocks new file creation)
    if payload.tool_name == "Write" && !resolved_path.exists() {
        for pattern in &config.pre_tool_use.prevent_additions {
            if matches_uneditable_pattern(
                &file_path,
                &relative_path,
                &resolved_path.to_string_lossy(),
                pattern,
            )? {
                let error_message = format!(
                    "Blocked {} operation: file matches preToolUse.preventAdditions pattern '{}'. File: {}",
                    payload.tool_name, pattern, file_path
                );

                eprintln!(
                    "PreToolUse blocked by preToolUse.preventAdditions pattern: tool_name={}, file_path={}, pattern={}",
                    payload.tool_name, file_path, pattern
                );

                return Ok(Some(HookResult::blocked(error_message)));
            }
        }
    }

    Ok(None)
}

/// Extract file path from tool input
pub fn extract_file_path<S: std::hash::BuildHasher>(
    tool_input: &std::collections::HashMap<String, Value, S>,
) -> Option<String> {
    tool_input
        .get("file_path")
        .or_else(|| tool_input.get("notebook_path"))
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string)
}

/// Extracts the Bash command string from tool input payload
/// Returns None if the command is missing, empty, or contains only whitespace
pub fn extract_bash_command<S: std::hash::BuildHasher>(
    tool_input: &std::collections::HashMap<String, Value, S>,
) -> Option<String> {
    tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(std::string::ToString::to_string)
}

/// Check if a file path represents a root addition
///
/// A file is considered a root addition if it's being created at the same directory
/// level as the .conclaude.yaml config file.
#[must_use]
pub fn is_root_addition(_file_path: &str, relative_path: &str, config_path: &Path) -> bool {
    // Handle edge cases - empty paths and parent directory references
    if relative_path.is_empty() || relative_path == ".." {
        return false;
    }

    // Get the directory containing the config file
    let config_dir = get_config_dir(config_path);

    // Get the current working directory
    let Ok(cwd) = std::env::current_dir() else {
        return false;
    };

    // Resolve the full path of the file being created
    let resolved_file_path = cwd.join(relative_path);

    // Get the directory that will contain the new file
    let file_parent_dir = resolved_file_path.parent().unwrap_or(&cwd);

    // Compare the canonical paths if possible, otherwise compare as-is
    let config_dir_canonical = config_dir
        .canonicalize()
        .unwrap_or_else(|_| config_dir.to_path_buf());
    let file_dir_canonical = file_parent_dir
        .canonicalize()
        .unwrap_or_else(|_| file_parent_dir.to_path_buf());

    // Block if the file is being created in the same directory as the config
    config_dir_canonical == file_dir_canonical
}

/// Check if a file matches an uneditable pattern
///
/// # Errors
///
/// Returns an error if the glob pattern is invalid.
pub fn matches_uneditable_pattern(
    file_path: &str,
    relative_path: &str,
    resolved_path: &str,
    pattern: &str,
) -> Result<bool> {
    let glob_pattern =
        Pattern::new(pattern).with_context(|| format!("Invalid glob pattern: {pattern}"))?;

    Ok(glob_pattern.matches(file_path)
        || glob_pattern.matches(relative_path)
        || glob_pattern.matches(resolved_path))
}

/// Check if an agent name matches a pattern.
/// Supports "*" wildcard and glob patterns.
#[must_use]
pub fn matches_agent_pattern(agent_name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    match Pattern::new(pattern) {
        Ok(glob_pattern) => glob_pattern.matches(agent_name),
        Err(e) => {
            eprintln!("Invalid agent pattern '{}': {}", pattern, e);
            false // Invalid pattern = no match (safe default)
        }
    }
}

/// Handles `PostToolUse` hook events fired after Claude executes a tool.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
#[allow(clippy::unused_async)]
pub async fn handle_post_tool_use() -> Result<HookResult> {
    let payload: PostToolUsePayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    if payload.tool_name.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: tool_name"));
    }

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing PostToolUse hook: session_id={}, tool_name={}",
        payload.base.session_id, payload.tool_name
    );

    // Send notification for post tool use completion
    send_notification(
        "PostToolUse",
        "success",
        Some(&format!("Tool '{}' completed", payload.tool_name)),
    );
    Ok(HookResult::success())
}

/// Handles `Notification` hook events when Claude sends system notifications.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
#[allow(clippy::unused_async)]
pub async fn handle_notification() -> Result<HookResult> {
    let payload: NotificationPayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    if payload.message.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: message"));
    }

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing Notification hook: session_id={}, message={}",
        payload.base.session_id, payload.message
    );

    // Send notification for notification hook processing
    send_notification(
        "Notification",
        "success",
        Some(&format!("Message: {}", payload.message)),
    );
    Ok(HookResult::success())
}

/// Expand @file references in a prompt string to the actual file contents.
///
/// References like @.claude/contexts/sidebar.md are replaced with the file contents.
/// Files are resolved relative to the config file directory.
/// Missing files are left as literal text and logged as warnings.
pub fn expand_file_references(prompt: &str, config_dir: &Path) -> String {
    use regex::Regex;

    // Match @path/to/file.ext pattern
    let re = Regex::new(r"@([\w\-./]+)").unwrap();

    re.replace_all(prompt, |caps: &regex::Captures| {
        let file_ref = &caps[1];
        let file_path = config_dir.join(file_ref);

        match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Warning: Failed to expand @{} reference: {}", file_ref, e);
                // Leave the reference as-is if file can't be read
                format!("@{}", file_ref)
            }
        }
    })
    .to_string()
}

/// Compile a context injection rule's pattern to a regex.
/// Returns None if the pattern fails to compile.
pub fn compile_rule_pattern(rule: &crate::config::ContextInjectionRule) -> Option<regex::Regex> {
    use regex::RegexBuilder;

    let pattern = if rule.case_insensitive.unwrap_or(false) {
        format!("(?i){}", rule.pattern)
    } else {
        rule.pattern.clone()
    };

    match RegexBuilder::new(&pattern).build() {
        Ok(regex) => Some(regex),
        Err(e) => {
            eprintln!(
                "Warning: Failed to compile context rule pattern '{}': {}",
                rule.pattern, e
            );
            None
        }
    }
}

/// Compile a command pattern to a regex.
/// Returns None if the pattern fails to compile or if no pattern is specified (matches all).
fn compile_command_pattern(command: &UserPromptSubmitCommand) -> Option<regex::Regex> {
    use regex::RegexBuilder;

    let Some(pattern) = &command.pattern else {
        return None; // No pattern means match all prompts
    };

    let full_pattern = if command.case_insensitive.unwrap_or(false) {
        format!("(?i){}", pattern)
    } else {
        pattern.clone()
    };

    match RegexBuilder::new(&full_pattern).build() {
        Ok(regex) => Some(regex),
        Err(e) => {
            eprintln!(
                "Warning: Failed to compile command pattern '{}': {}",
                pattern, e
            );
            None
        }
    }
}

/// Build environment variables for user prompt submit command execution
///
/// Creates a HashMap of environment variables to pass to commands, including
/// user prompt and session information.
///
/// # Arguments
///
/// * `payload` - The UserPromptSubmitPayload containing prompt information
/// * `config_dir` - The directory containing the configuration file
#[must_use]
pub(crate) fn build_user_prompt_submit_env_vars(
    payload: &UserPromptSubmitPayload,
    config_dir: &Path,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    // User prompt environment variable (empty string if nil)
    let prompt_text = payload.prompt.as_deref().unwrap_or("");
    env_vars.insert("CONCLAUDE_USER_PROMPT".to_string(), prompt_text.to_string());

    // Session-level environment variables
    env_vars.insert(
        "CONCLAUDE_SESSION_ID".to_string(),
        payload.base.session_id.clone(),
    );
    env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
    env_vars.insert(
        "CONCLAUDE_CONFIG_DIR".to_string(),
        config_dir.to_string_lossy().to_string(),
    );
    env_vars.insert(
        "CONCLAUDE_HOOK_EVENT".to_string(),
        "UserPromptSubmit".to_string(),
    );

    env_vars
}

/// Collect user prompt submit commands from configuration that match the given prompt
///
/// Returns commands that either have no pattern (match all) or whose pattern matches the prompt.
///
/// # Errors
///
/// Returns an error if bash command extraction fails.
pub(crate) fn collect_user_prompt_submit_commands(
    commands: &[UserPromptSubmitCommand],
    prompt: &str,
) -> Result<Vec<UserPromptSubmitCommandConfig>> {
    let mut result = Vec::new();

    for cmd_config in commands {
        // Check if command should run for this prompt
        let should_run = match compile_command_pattern(cmd_config) {
            Some(regex) => regex.is_match(prompt),
            None => true, // No pattern means run for all prompts
        };

        if !should_run {
            continue;
        }

        // Extract and add commands
        let extracted = extract_bash_commands(&cmd_config.run)?;
        let show_stdout = cmd_config.show_stdout.unwrap_or(false);
        let show_stderr = cmd_config.show_stderr.unwrap_or(false);
        let show_command = cmd_config.show_command.unwrap_or(true);
        let max_output_lines = cmd_config.max_output_lines;
        let timeout = cmd_config.timeout;
        let notify_per_command = cmd_config.notify_per_command.unwrap_or(false);

        for cmd in extracted {
            result.push(UserPromptSubmitCommandConfig {
                command: cmd,
                show_stdout,
                show_stderr,
                max_output_lines,
                timeout,
                show_command,
                notify_per_command,
            });
        }
    }

    Ok(result)
}

/// Execute user prompt submit hook commands with environment variables
///
/// Commands are observational (read-only) and cannot block prompt processing.
/// Failures are logged but do not affect the hook result.
///
/// # Errors
///
/// Returns an error if command spawning fails. Individual command failures are logged
/// but do not stop subsequent command execution.
async fn execute_user_prompt_submit_commands(
    commands: &[UserPromptSubmitCommandConfig],
    env_vars: &HashMap<String, String>,
    config_dir: &Path,
) -> Result<()> {
    if commands.is_empty() {
        return Ok(());
    }

    println!(
        "Executing {} user prompt submit hook commands",
        commands.len()
    );

    for (index, cmd_config) in commands.iter().enumerate() {
        if cmd_config.show_command {
            println!(
                "Executing user prompt submit command {}/{}: {}",
                index + 1,
                commands.len(),
                cmd_config.command
            );
        } else {
            println!(
                "Executing user prompt submit command {}/{}",
                index + 1,
                commands.len()
            );
        }

        // Send start notification if per-command notifications are enabled
        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Running: {}", cmd_config.command)
            } else {
                "Running command".to_string()
            };
            send_notification("UserPromptSubmit", "running", Some(&context_msg));
        }

        let child = TokioCommand::new("bash")
            .arg("-c")
            .arg(&cmd_config.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env_vars)
            .current_dir(config_dir)
            .spawn();

        let child = match child {
            Ok(c) => c,
            Err(e) => {
                // Log error but continue to next command
                if cmd_config.show_command {
                    eprintln!(
                        "Failed to spawn user prompt submit command '{}': {}",
                        cmd_config.command, e
                    );
                } else {
                    eprintln!("Failed to spawn user prompt submit command: {}", e);
                }

                // Send failure notification if per-command notifications are enabled
                if cmd_config.notify_per_command {
                    let context_msg = if cmd_config.show_command {
                        format!("Failed to spawn command: {}", cmd_config.command)
                    } else {
                        "Failed to spawn command".to_string()
                    };
                    send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                }

                continue;
            }
        };

        let output = if let Some(timeout_secs) = cmd_config.timeout {
            match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(result) => match result {
                    Ok(o) => o,
                    Err(e) => {
                        if cmd_config.show_command {
                            eprintln!(
                                "Failed to wait for user prompt submit command '{}': {}",
                                cmd_config.command, e
                            );
                        } else {
                            eprintln!("Failed to wait for user prompt submit command: {}", e);
                        }

                        // Send failure notification if per-command notifications are enabled
                        if cmd_config.notify_per_command {
                            let context_msg = if cmd_config.show_command {
                                format!("Command failed to wait: {}", cmd_config.command)
                            } else {
                                "Command failed to wait".to_string()
                            };
                            send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                        }

                        continue;
                    }
                },
                Err(_) => {
                    // Timeout occurred - log and continue
                    if cmd_config.show_command {
                        eprintln!(
                            "User prompt submit command timed out after {} seconds: {}",
                            timeout_secs, cmd_config.command
                        );
                    } else {
                        eprintln!(
                            "User prompt submit command timed out after {} seconds",
                            timeout_secs
                        );
                    }

                    // Send failure notification if per-command notifications are enabled
                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command timed out: {}", cmd_config.command)
                        } else {
                            "Command timed out".to_string()
                        };
                        send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        } else {
            match child.wait_with_output().await {
                Ok(o) => o,
                Err(e) => {
                    if cmd_config.show_command {
                        eprintln!(
                            "Failed to wait for user prompt submit command '{}': {}",
                            cmd_config.command, e
                        );
                    } else {
                        eprintln!("Failed to wait for user prompt submit command: {}", e);
                    }

                    // Send failure notification if per-command notifications are enabled
                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command failed to wait: {}", cmd_config.command)
                        } else {
                            "Command failed to wait".to_string()
                        };
                        send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(1);

            // Log failure information - respect showCommand flag
            let mut diagnostic = if cmd_config.show_command {
                format!(
                    "User prompt submit command failed:\n  Command: {}\n  Status: Failed (exit code: {})",
                    cmd_config.command, exit_code
                )
            } else {
                format!(
                    "User prompt submit command failed:\n  Status: Failed (exit code: {})",
                    exit_code
                )
            };

            if cmd_config.show_stdout && !stdout.trim().is_empty() {
                let stdout_content = if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                    if is_truncated {
                        format!("{}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        truncated
                    }
                } else {
                    stdout.trim().to_string()
                };
                let stdout_display = stdout_content
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                diagnostic.push_str(&format!("\n  Stdout:\n{}", stdout_display));
            }

            if cmd_config.show_stderr && !stderr.trim().is_empty() {
                let stderr_content = if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                    if is_truncated {
                        format!("{}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        truncated
                    }
                } else {
                    stderr.trim().to_string()
                };
                let stderr_display = stderr_content
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                diagnostic.push_str(&format!("\n  Stderr:\n{}", stderr_display));
            }

            eprintln!("{}", diagnostic);

            // Send failure notification if per-command notifications are enabled
            if cmd_config.notify_per_command {
                let context_msg = if cmd_config.show_command {
                    format!("Command failed: {}", cmd_config.command)
                } else {
                    "Command failed".to_string()
                };
                send_notification("UserPromptSubmit", "failure", Some(&context_msg));
            }

            // Continue to next command (graceful failure handling)
            continue;
        }

        // Successful command - show output if configured
        if cmd_config.show_stdout && !stdout.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stdout.to_string()
            };
            println!("Stdout: {}", output_to_show);
        }

        if cmd_config.show_stderr && !stderr.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stderr.to_string()
            };
            eprintln!("Stderr: {}", output_to_show);
        }

        // Send success notification if per-command notifications are enabled
        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Command completed: {}", cmd_config.command)
            } else {
                "Command completed".to_string()
            };
            send_notification("UserPromptSubmit", "success", Some(&context_msg));
        }
    }

    println!("All user prompt submit hook commands completed");
    Ok(())
}

/// Handles `UserPromptSubmit` hook events when users submit input to Claude.
///
/// This function processes user prompt submissions by:
/// 1. Validating the payload
/// 2. Evaluating contextRules for context injection
/// 3. Executing matching commands (after contextRules processing)
/// 4. Returning the hook result with any injected context
///
/// Commands are observational (read-only) and cannot block prompt processing.
/// Command failures are logged but do not affect the hook result.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
pub async fn handle_user_prompt_submit() -> Result<HookResult> {
    let payload: UserPromptSubmitPayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing UserPromptSubmit hook: session_id={}",
        payload.base.session_id
    );

    // Load configuration for context injection rules and commands
    let (config, config_path) = get_config().await?;
    let config_dir = get_config_dir(config_path);

    // Check if any context injection rules match
    let mut matching_contexts = Vec::new();
    let mut matched_patterns = Vec::new();

    // Only evaluate context rules if prompt is present and non-empty
    if let Some(ref prompt) = payload.prompt {
        if !prompt.is_empty() {
            for rule in &config.user_prompt_submit.context_rules {
                // Skip disabled rules
                if rule.enabled == Some(false) {
                    continue;
                }

                // Compile the pattern
                let Some(regex) = compile_rule_pattern(rule) else {
                    continue;
                };

                // Check if the pattern matches the user prompt
                if regex.is_match(prompt) {
                    // Expand any @file references in the prompt
                    let expanded_prompt = expand_file_references(&rule.prompt, config_dir);
                    matching_contexts.push(expanded_prompt);
                    matched_patterns.push(rule.pattern.clone());
                }
            }
        }
    }

    // Determine context injection result before executing commands
    let context_result = if !matching_contexts.is_empty() {
        let combined_context = matching_contexts.join("\n\n");
        println!(
            "Context injection: {} rule(s) matched user prompt",
            matching_contexts.len()
        );
        println!("Matched patterns: {:?}", matched_patterns);
        Some(combined_context)
    } else {
        None
    };

    // Execute commands after contextRules processing
    // Commands are observational and cannot block prompt processing
    if !config.user_prompt_submit.commands.is_empty() {
        // Only execute commands if prompt is present and non-empty
        if let Some(ref prompt) = payload.prompt {
            if !prompt.is_empty() {
                // Collect commands that match the prompt
                let commands = collect_user_prompt_submit_commands(
                    &config.user_prompt_submit.commands,
                    prompt,
                )?;

                if !commands.is_empty() {
                    // Build environment variables
                    let env_vars = build_user_prompt_submit_env_vars(&payload, config_dir);

                    // Execute commands (graceful failure handling)
                    if let Err(e) =
                        execute_user_prompt_submit_commands(&commands, &env_vars, config_dir).await
                    {
                        // Log error but don't block the hook result
                        eprintln!("Error executing user prompt submit commands: {}", e);
                    }
                }
            }
        }
    }

    // Execute slash command hooks if configured and a slash command is detected
    if let Some(ref slash_config) = config.user_prompt_submit.slash_commands {
        if !slash_config.commands.is_empty() {
            if let Some(ref prompt) = payload.prompt {
                if let Some(detection) = detect_slash_command(prompt) {
                    println!(
                        "Detected slash command: /{} with args: '{}'",
                        detection.command, detection.args
                    );

                    // Match the command against configured patterns
                    let matching_patterns =
                        match_slash_command_patterns(&detection.command, slash_config)?;

                    if !matching_patterns.is_empty() {
                        println!(
                            "Executing slash command hooks for '/{}' (matched {} pattern(s))",
                            detection.command,
                            matching_patterns.len()
                        );

                        // Collect commands from matching patterns
                        let commands =
                            collect_slash_command_entries(slash_config, &matching_patterns)?;

                        if !commands.is_empty() {
                            // Build environment variables with slash command context
                            let env_vars =
                                build_slash_command_env_vars(&payload, config_dir, &detection);

                            // Execute slash command hooks (can block on exit code 2)
                            match execute_slash_command_hooks(&commands, &env_vars, config_dir)
                                .await
                            {
                                Ok(blocked) => {
                                    if let Some(message) = blocked {
                                        return Ok(HookResult::blocked(message));
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error executing slash command hooks: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Return the hook result with context if any rules matched
    if let Some(context) = context_result {
        send_notification(
            "UserPromptSubmit",
            "success",
            Some(&format!(
                "Context injected ({} rule(s) matched)",
                matching_contexts.len()
            )),
        );
        return Ok(HookResult::with_context(context));
    }

    // Send notification for user prompt submission (no context injection)
    send_notification("UserPromptSubmit", "success", Some("User input received"));
    Ok(HookResult::success())
}

/// Handles `SessionStart` hook events when a new Claude session begins.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
#[allow(clippy::unused_async)]
pub async fn handle_session_start() -> Result<HookResult> {
    let payload: SessionStartPayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    if payload.source.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: source"));
    }

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing SessionStart hook: session_id={}, source={}",
        payload.base.session_id, payload.source
    );

    // Send notification for session start
    send_notification(
        "SessionStart",
        "success",
        Some(&format!("Session started from {}", payload.source)),
    );
    Ok(HookResult::success())
}

/// Handles `SessionEnd` hook events when a Claude session terminates.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
#[allow(clippy::unused_async)]
pub async fn handle_session_end() -> Result<HookResult> {
    let payload: SessionEndPayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    if payload.reason.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: reason"));
    }

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing SessionEnd hook: session_id={}, reason={}",
        payload.base.session_id, payload.reason
    );

    Ok(HookResult::success())
}

/// Truncate output to a maximum number of lines
///
/// Returns a tuple of (truncated_output, is_truncated, omitted_line_count)
pub(crate) fn truncate_output(output: &str, max_lines: u32) -> (String, bool, usize) {
    let lines: Vec<&str> = output.lines().collect();
    let total_lines = lines.len();
    let max_lines_usize = max_lines as usize;

    if total_lines <= max_lines_usize {
        // No truncation needed
        (output.to_string(), false, 0)
    } else {
        // Take first N lines and calculate omitted count
        let truncated_lines = &lines[..max_lines_usize];
        let omitted_count = total_lines - max_lines_usize;
        let truncated = truncated_lines.join("\n");
        (truncated, true, omitted_count)
    }
}

/// Collect stop commands from configuration
///
/// # Errors
///
/// Returns an error if bash command extraction fails.
pub(crate) fn collect_stop_commands(config: &ConclaudeConfig) -> Result<Vec<StopCommandConfig>> {
    let mut commands = Vec::new();

    // Add structured commands with messages and output control
    for cmd_config in &config.stop.commands {
        let extracted = extract_bash_commands(&cmd_config.run)?;
        let show_stdout = cmd_config.show_stdout.unwrap_or(false);
        let show_stderr = cmd_config.show_stderr.unwrap_or(false);
        let show_command = cmd_config.show_command.unwrap_or(true);
        let max_output_lines = cmd_config.max_output_lines;
        let notify_per_command = cmd_config.notify_per_command.unwrap_or(false);
        for cmd in extracted {
            commands.push(StopCommandConfig {
                command: cmd,
                message: cmd_config.message.clone(),
                show_stdout,
                show_stderr,
                max_output_lines,
                timeout: cmd_config.timeout,
                show_command,
                notify_per_command,
            });
        }
    }

    Ok(commands)
}

/// Execute stop hook commands
///
/// # Errors
///
/// Returns an error if command execution fails or process spawning fails.
async fn execute_stop_commands(
    commands: &[StopCommandConfig],
    config_dir: &Path,
) -> Result<Option<HookResult>> {
    println!("Executing {} stop hook commands", commands.len());

    for (index, cmd_config) in commands.iter().enumerate() {
        if cmd_config.show_command {
            println!(
                "Executing command {}/{}: {}",
                index + 1,
                commands.len(),
                cmd_config.command
            );
        } else {
            println!("Executing command {}/{}", index + 1, commands.len());
        }

        // Send start notification if per-command notifications are enabled
        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Running: {}", cmd_config.command)
            } else {
                "Running command".to_string()
            };
            send_notification("Stop", "running", Some(&context_msg));
        }

        let child = TokioCommand::new("bash")
            .arg("-c")
            .arg(&cmd_config.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(config_dir)
            .env(
                "CONCLAUDE_CONFIG_DIR",
                config_dir.to_string_lossy().to_string(),
            )
            .spawn()
            .with_context(|| format!("Failed to spawn command: {}", cmd_config.command))?;

        let output = if let Some(timeout_secs) = cmd_config.timeout {
            match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(result) => result.with_context(|| {
                    format!("Failed to wait for command: {}", cmd_config.command)
                })?,
                Err(_) => {
                    // Timeout occurred - return blocked result
                    let error_msg = format!(
                        "Command timed out after {} seconds: {}",
                        timeout_secs, cmd_config.command
                    );
                    eprintln!("{}", error_msg);

                    // Send failure notification if per-command notifications are enabled
                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command timed out: {}", cmd_config.command)
                        } else {
                            "Command timed out".to_string()
                        };
                        send_notification("Stop", "failure", Some(&context_msg));
                    }

                    let message = cmd_config.message.as_deref().unwrap_or(&error_msg);
                    return Ok(Some(HookResult::blocked(message)));
                }
            }
        } else {
            child
                .wait_with_output()
                .await
                .with_context(|| format!("Failed to wait for command: {}", cmd_config.command))?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(1);

            // Log detailed failure information with command and outputs appended
            // Respect showCommand, showStdout and showStderr flags when logging to console
            // Build diagnostic output dynamically to omit sections when flags are false
            let mut diagnostic = if cmd_config.show_command {
                format!(
                    "Stop command failed:\n  Command: {}\n  Status: Failed (exit code: {})",
                    cmd_config.command, exit_code
                )
            } else {
                format!(
                    "Stop command failed:\n  Status: Failed (exit code: {})",
                    exit_code
                )
            };

            // Only include Stdout section if showStdout is true
            if cmd_config.show_stdout && !stdout.trim().is_empty() {
                let stdout_content = if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                    if is_truncated {
                        format!("{}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        truncated
                    }
                } else {
                    stdout.trim().to_string()
                };
                let stdout_display = stdout_content
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                diagnostic.push_str(&format!("\n  Stdout:\n{}", stdout_display));
            }

            // Only include Stderr section if showStderr is true
            if cmd_config.show_stderr && !stderr.trim().is_empty() {
                let stderr_content = if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                    if is_truncated {
                        format!("{}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        truncated
                    }
                } else {
                    stderr.trim().to_string()
                };
                let stderr_display = stderr_content
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                diagnostic.push_str(&format!("\n  Stderr:\n{}", stderr_display));
            }

            eprintln!("{}", diagnostic);

            let stdout_section = if cmd_config.show_stdout && !stdout.is_empty() {
                if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                    if is_truncated {
                        format!("\nStdout: {}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        format!("\nStdout: {}", truncated)
                    }
                } else {
                    format!("\nStdout: {}", stdout)
                }
            } else {
                String::new()
            };

            let stderr_section = if cmd_config.show_stderr && !stderr.is_empty() {
                if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                    if is_truncated {
                        format!("\nStderr: {}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        format!("\nStderr: {}", truncated)
                    }
                } else {
                    format!("\nStderr: {}", stderr)
                }
            } else {
                String::new()
            };

            let error_message = if let Some(custom_msg) = &cmd_config.message {
                format!("{custom_msg}{stdout_section}{stderr_section}")
            } else if cmd_config.show_command {
                format!(
                    "Command failed with exit code {exit_code}: {}{stdout_section}{stderr_section}",
                    cmd_config.command
                )
            } else {
                format!("Command failed with exit code {exit_code}{stdout_section}{stderr_section}")
            };

            // Send failure notification if per-command notifications are enabled
            if cmd_config.notify_per_command {
                let context_msg = if cmd_config.show_command {
                    format!("Command failed: {}", cmd_config.command)
                } else {
                    "Command failed".to_string()
                };
                send_notification("Stop", "failure", Some(&context_msg));
            }

            return Ok(Some(HookResult::blocked(error_message)));
        }

        // Send success notification if per-command notifications are enabled
        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Command completed: {}", cmd_config.command)
            } else {
                "Command completed".to_string()
            };
            send_notification("Stop", "success", Some(&context_msg));
        }

        // Successful individual commands produce no output
    }

    println!("All stop hook commands completed successfully");
    Ok(None)
}

/// Handles `Stop` hook events when a Claude session is terminating.
///
/// # Errors
///
/// Returns an error if payload validation fails, configuration loading fails,
/// command execution fails, or directory operations fail.
pub async fn handle_stop() -> Result<HookResult> {
    let payload: StopPayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing Stop hook: session_id={}",
        payload.base.session_id
    );

    let (config, config_path) = get_config().await?;
    let config_dir = get_config_dir(config_path);

    // Snapshot root directory if preventRootAdditions is enabled
    let root_snapshot = if config.pre_tool_use.prevent_root_additions {
        Some(snapshot_root_directory()?)
    } else {
        None
    };

    // Extract and execute commands from config.stop.commands
    let commands_with_messages = collect_stop_commands(config)?;

    // Execute commands
    if let Some(result) = execute_stop_commands(&commands_with_messages, config_dir).await? {
        // Send notification for blocked/failed stop hook
        send_notification(
            "Stop",
            "failure",
            Some(
                &result
                    .message
                    .clone()
                    .unwrap_or_else(|| "Hook blocked".to_string()),
            ),
        );
        return Ok(result);
    }

    // Check root additions if enabled
    if let Some(snapshot) = root_snapshot {
        if let Some(result) = check_root_additions(&snapshot)? {
            // Send notification for blocked root additions
            send_notification(
                "Stop",
                "failure",
                Some(
                    &result
                        .message
                        .clone()
                        .unwrap_or_else(|| "Root additions blocked".to_string()),
                ),
            );
            return Ok(result);
        }
    }

    // Check if infinite mode is enabled
    if config.stop.infinite {
        let infinite_message = config
            .stop
            .infinite_message
            .as_deref()
            .unwrap_or("continue working on the task");

        println!("Infinite mode enabled, sending continuation message: {infinite_message}");
        // Send notification for infinite mode continuation
        send_notification(
            "Stop",
            "success",
            Some(&format!("Continuing: {}", infinite_message)),
        );
        return Ok(HookResult::blocked(infinite_message.to_string()));
    }

    // Send notification for successful stop hook completion
    send_notification("Stop", "success", None);
    Ok(HookResult::success())
}

/// Handles `SubagentStart` hook events when Claude subagents begin execution.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
#[allow(clippy::unused_async)]
pub async fn handle_subagent_start() -> Result<HookResult> {
    let payload: SubagentStartPayload = read_payload_from_stdin()?;

    // Validate the payload including agent_id, subagent_type, and agent_transcript_path fields
    validate_subagent_start_payload(&payload).map_err(|e| anyhow::anyhow!(e))?;

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing SubagentStart hook: session_id={}, agent_id={}",
        payload.base.session_id, payload.agent_id
    );

    // Set environment variables for the subagent's information
    // These allow downstream hooks and processes to access subagent details
    std::env::set_var("CONCLAUDE_AGENT_ID", &payload.agent_id);
    std::env::set_var("CONCLAUDE_SUBAGENT_TYPE", &payload.subagent_type);
    std::env::set_var(
        "CONCLAUDE_AGENT_TRANSCRIPT_PATH",
        &payload.agent_transcript_path,
    );

    // Load configuration and execute skill start commands if configured
    let config_result = get_config().await;
    if let Ok((config, config_path)) = config_result {
        let config_dir = get_config_dir(config_path);

        // Check if skillStart commands are configured
        if !config.skill_start.commands.is_empty() {
            // Match subagent_type against configured skill patterns
            let matching_patterns =
                match_skill_patterns(&payload.subagent_type, &config.skill_start)?;

            if !matching_patterns.is_empty() {
                println!(
                    "Executing skill start commands for skill '{}' (matched {} pattern(s))",
                    payload.subagent_type,
                    matching_patterns.len()
                );

                // Collect commands from matching patterns
                let commands =
                    collect_skill_start_commands(&config.skill_start, &matching_patterns)?;

                if !commands.is_empty() {
                    // Build environment variables for command execution
                    let env_vars = build_skill_start_env_vars(&payload, config_dir);

                    // Execute skill start commands
                    if let Err(e) =
                        execute_skill_start_commands(&commands, &env_vars, config_dir).await
                    {
                        eprintln!("Error executing skill start commands: {}", e);
                    }
                }
            }
        }
    }

    // Send notification for subagent start with agent ID included
    send_notification(
        "SubagentStart",
        "success",
        Some(&format!("Subagent '{}' started", payload.agent_id)),
    );
    Ok(HookResult::success())
}

/// Match agent_id against configured patterns in SubagentStopConfig
///
/// Returns a vector of matched pattern strings, with wildcard "*" first (if matched),
/// followed by other matching patterns in stable order.
///
/// # Arguments
///
/// * `agent_id` - The agent identifier to match against patterns
/// * `config` - The subagent stop configuration containing pattern mappings
///
/// # Errors
///
/// Returns an error if a glob pattern in the configuration is invalid.
pub(crate) fn match_subagent_patterns<'a>(
    agent_id: &str,
    config: &'a SubagentStopConfig,
) -> Result<Vec<&'a str>> {
    let mut wildcard_matches = Vec::new();
    let mut other_matches = Vec::new();

    for pattern_str in config.commands.keys() {
        // Handle wildcard pattern specially - it always matches
        if pattern_str == "*" {
            wildcard_matches.push(pattern_str.as_str());
            continue;
        }

        // Use glob pattern matching for other patterns
        let pattern = Pattern::new(pattern_str).with_context(|| {
            format!(
                "Invalid glob pattern in subagentStop config: {}",
                pattern_str
            )
        })?;

        if pattern.matches(agent_id) {
            other_matches.push(pattern_str.as_str());
        }
    }

    // Sort non-wildcard matches for consistent ordering
    other_matches.sort();

    // Wildcard first, then sorted other matches
    let mut result = wildcard_matches;
    result.extend(other_matches);
    Ok(result)
}

/// Build environment variables for subagent stop command execution
///
/// Creates a HashMap of environment variables to pass to commands, including
/// subagent context and session information.
///
/// # Arguments
///
/// * `payload` - The SubagentStopPayload containing subagent information
/// * `config_dir` - The directory containing the configuration file
/// * `agent_name` - The extracted agent name (subagent_type), if available
#[must_use]
pub(crate) fn build_subagent_env_vars(
    payload: &SubagentStopPayload,
    config_dir: &Path,
    agent_name: Option<&str>,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    // Agent-specific environment variables
    env_vars.insert("CONCLAUDE_AGENT_ID".to_string(), payload.agent_id.clone());
    env_vars.insert(
        "CONCLAUDE_AGENT_TRANSCRIPT_PATH".to_string(),
        payload.agent_transcript_path.clone(),
    );

    // Set CONCLAUDE_AGENT_NAME to the extracted name, or fall back to agent_id
    let agent_name_value = agent_name.unwrap_or(&payload.agent_id);
    env_vars.insert(
        "CONCLAUDE_AGENT_NAME".to_string(),
        agent_name_value.to_string(),
    );

    // Session-level environment variables
    env_vars.insert(
        "CONCLAUDE_SESSION_ID".to_string(),
        payload.base.session_id.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_TRANSCRIPT_PATH".to_string(),
        payload.base.transcript_path.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_HOOK_EVENT".to_string(),
        "SubagentStop".to_string(),
    );
    env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
    env_vars.insert(
        "CONCLAUDE_CONFIG_DIR".to_string(),
        config_dir.to_string_lossy().to_string(),
    );

    // Full JSON payload for advanced use cases
    if let Ok(json) = serde_json::to_string(payload) {
        env_vars.insert("CONCLAUDE_PAYLOAD_JSON".to_string(), json);
    }

    env_vars
}

/// Collect subagent stop commands from configuration for matching patterns
///
/// # Errors
///
/// Returns an error if bash command extraction fails.
pub(crate) fn collect_subagent_stop_commands(
    config: &SubagentStopConfig,
    matching_patterns: &[&str],
) -> Result<Vec<SubagentStopCommandConfig>> {
    let mut commands = Vec::new();

    for pattern in matching_patterns {
        if let Some(cmd_list) = config.commands.get(*pattern) {
            for cmd_config in cmd_list {
                let extracted = extract_bash_commands(&cmd_config.run)?;
                let show_stdout = cmd_config.show_stdout.unwrap_or(false);
                let show_stderr = cmd_config.show_stderr.unwrap_or(false);
                let show_command = cmd_config.show_command.unwrap_or(true);
                let max_output_lines = cmd_config.max_output_lines;
                let notify_per_command = cmd_config.notify_per_command.unwrap_or(false);

                for cmd in extracted {
                    commands.push(SubagentStopCommandConfig {
                        command: cmd,
                        message: cmd_config.message.clone(),
                        show_stdout,
                        show_stderr,
                        max_output_lines,
                        timeout: cmd_config.timeout,
                        show_command,
                        notify_per_command,
                    });
                }
            }
        }
    }

    Ok(commands)
}

/// Execute subagent stop hook commands with environment variables
///
/// # Errors
///
/// Returns an error if command spawning fails. Individual command failures are logged
/// but do not stop subsequent command execution.
async fn execute_subagent_stop_commands(
    commands: &[SubagentStopCommandConfig],
    env_vars: &HashMap<String, String>,
    config_dir: &Path,
) -> Result<()> {
    if commands.is_empty() {
        return Ok(());
    }

    println!("Executing {} subagent stop hook commands", commands.len());

    for (index, cmd_config) in commands.iter().enumerate() {
        if cmd_config.show_command {
            println!(
                "Executing subagent stop command {}/{}: {}",
                index + 1,
                commands.len(),
                cmd_config.command
            );
        } else {
            println!(
                "Executing subagent stop command {}/{}",
                index + 1,
                commands.len()
            );
        }

        // Send start notification if per-command notifications are enabled
        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Running: {}", cmd_config.command)
            } else {
                "Running command".to_string()
            };
            send_notification("SubagentStop", "running", Some(&context_msg));
        }

        let child = TokioCommand::new("bash")
            .arg("-c")
            .arg(&cmd_config.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env_vars)
            .current_dir(config_dir)
            .spawn();

        let child = match child {
            Ok(c) => c,
            Err(e) => {
                // Log error but continue to next command
                if cmd_config.show_command {
                    eprintln!(
                        "Failed to spawn subagent stop command '{}': {}",
                        cmd_config.command, e
                    );
                } else {
                    eprintln!("Failed to spawn subagent stop command: {}", e);
                }

                // Send failure notification if per-command notifications are enabled
                if cmd_config.notify_per_command {
                    let context_msg = if cmd_config.show_command {
                        format!("Failed to spawn command: {}", cmd_config.command)
                    } else {
                        "Failed to spawn command".to_string()
                    };
                    send_notification("SubagentStop", "failure", Some(&context_msg));
                }

                continue;
            }
        };

        let output = if let Some(timeout_secs) = cmd_config.timeout {
            match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(result) => match result {
                    Ok(o) => o,
                    Err(e) => {
                        // Log error but continue to next command
                        if cmd_config.show_command {
                            eprintln!(
                                "Failed to wait for subagent stop command '{}': {}",
                                cmd_config.command, e
                            );
                        } else {
                            eprintln!("Failed to wait for subagent stop command: {}", e);
                        }

                        // Send failure notification if per-command notifications are enabled
                        if cmd_config.notify_per_command {
                            let context_msg = if cmd_config.show_command {
                                format!("Command failed to wait: {}", cmd_config.command)
                            } else {
                                "Command failed to wait".to_string()
                            };
                            send_notification("SubagentStop", "failure", Some(&context_msg));
                        }

                        continue;
                    }
                },
                Err(_) => {
                    // Timeout occurred - log and continue
                    // Note: child is consumed by wait_with_output, so we can't kill it here
                    if cmd_config.show_command {
                        eprintln!(
                            "Subagent stop command timed out after {} seconds: {}",
                            timeout_secs, cmd_config.command
                        );
                    } else {
                        eprintln!(
                            "Subagent stop command timed out after {} seconds",
                            timeout_secs
                        );
                    }
                    if let Some(custom_msg) = &cmd_config.message {
                        eprintln!("Message: {}", custom_msg);
                    }

                    // Send failure notification if per-command notifications are enabled
                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command timed out: {}", cmd_config.command)
                        } else {
                            "Command timed out".to_string()
                        };
                        send_notification("SubagentStop", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        } else {
            match child.wait_with_output().await {
                Ok(o) => o,
                Err(e) => {
                    // Log error but continue to next command
                    if cmd_config.show_command {
                        eprintln!(
                            "Failed to wait for subagent stop command '{}': {}",
                            cmd_config.command, e
                        );
                    } else {
                        eprintln!("Failed to wait for subagent stop command: {}", e);
                    }

                    // Send failure notification if per-command notifications are enabled
                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command failed to wait: {}", cmd_config.command)
                        } else {
                            "Command failed to wait".to_string()
                        };
                        send_notification("SubagentStop", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(1);

            // Log failure information
            // Respect showCommand flag when displaying command
            let mut diagnostic = if cmd_config.show_command {
                format!(
                    "Subagent stop command failed:\n  Command: {}\n  Status: Failed (exit code: {})",
                    cmd_config.command, exit_code
                )
            } else {
                format!(
                    "Subagent stop command failed:\n  Status: Failed (exit code: {})",
                    exit_code
                )
            };

            if cmd_config.show_stdout && !stdout.trim().is_empty() {
                let stdout_content = if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                    if is_truncated {
                        format!("{}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        truncated
                    }
                } else {
                    stdout.trim().to_string()
                };
                let stdout_display = stdout_content
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                diagnostic.push_str(&format!("\n  Stdout:\n{}", stdout_display));
            }

            if cmd_config.show_stderr && !stderr.trim().is_empty() {
                let stderr_content = if let Some(max_lines) = cmd_config.max_output_lines {
                    let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                    if is_truncated {
                        format!("{}\n... ({} lines omitted)", truncated, omitted)
                    } else {
                        truncated
                    }
                } else {
                    stderr.trim().to_string()
                };
                let stderr_display = stderr_content
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                diagnostic.push_str(&format!("\n  Stderr:\n{}", stderr_display));
            }

            eprintln!("{}", diagnostic);

            // If there's a custom message, print it
            if let Some(custom_msg) = &cmd_config.message {
                eprintln!("Message: {}", custom_msg);
            }

            // Send failure notification if per-command notifications are enabled
            if cmd_config.notify_per_command {
                let context_msg = if cmd_config.show_command {
                    format!("Command failed: {}", cmd_config.command)
                } else {
                    "Command failed".to_string()
                };
                send_notification("SubagentStop", "failure", Some(&context_msg));
            }

            // Continue to next command (graceful failure handling)
            continue;
        }

        // Successful command - show output if configured
        if cmd_config.show_stdout && !stdout.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stdout.to_string()
            };
            println!("Stdout: {}", output_to_show);
        }

        if cmd_config.show_stderr && !stderr.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stderr.to_string()
            };
            eprintln!("Stderr: {}", output_to_show);
        }

        // Send success notification if per-command notifications are enabled
        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Command completed: {}", cmd_config.command)
            } else {
                "Command completed".to_string()
            };
            send_notification("SubagentStop", "success", Some(&context_msg));
        }
    }

    println!("All subagent stop hook commands completed");
    Ok(())
}

/// Match skill name against configured patterns in SkillStartConfig
///
/// Returns a vector of matched pattern strings, with wildcard "*" first (if matched),
/// followed by other matching patterns in stable order.
///
/// # Arguments
///
/// * `skill_name` - The skill/subagent type name to match against patterns
/// * `config` - The skill start configuration containing pattern mappings
///
/// # Errors
///
/// Returns an error if a glob pattern in the configuration is invalid.
pub(crate) fn match_skill_patterns<'a>(
    skill_name: &str,
    config: &'a SkillStartConfig,
) -> Result<Vec<&'a str>> {
    let mut wildcard_matches = Vec::new();
    let mut other_matches = Vec::new();

    for pattern_str in config.commands.keys() {
        // Handle wildcard pattern specially - it always matches
        if pattern_str == "*" {
            wildcard_matches.push(pattern_str.as_str());
            continue;
        }

        // Use glob pattern matching for other patterns
        let pattern = Pattern::new(pattern_str).with_context(|| {
            format!("Invalid glob pattern in skillStart config: {}", pattern_str)
        })?;

        if pattern.matches(skill_name) {
            other_matches.push(pattern_str.as_str());
        }
    }

    // Sort non-wildcard matches for consistent ordering
    other_matches.sort();

    // Wildcard first, then sorted other matches
    let mut result = wildcard_matches;
    result.extend(other_matches);
    Ok(result)
}

/// Build environment variables for skill start command execution
///
/// Creates a HashMap of environment variables to pass to commands, including
/// skill context and session information.
///
/// # Arguments
///
/// * `payload` - The SubagentStartPayload containing skill information
/// * `config_dir` - The directory containing the configuration file
#[must_use]
pub(crate) fn build_skill_start_env_vars(
    payload: &SubagentStartPayload,
    config_dir: &Path,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    // Skill-specific environment variables
    env_vars.insert(
        "CONCLAUDE_SKILL_NAME".to_string(),
        payload.subagent_type.clone(),
    );
    env_vars.insert("CONCLAUDE_AGENT_ID".to_string(), payload.agent_id.clone());
    env_vars.insert(
        "CONCLAUDE_AGENT_TRANSCRIPT_PATH".to_string(),
        payload.agent_transcript_path.clone(),
    );

    // Session-level environment variables
    env_vars.insert(
        "CONCLAUDE_SESSION_ID".to_string(),
        payload.base.session_id.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_TRANSCRIPT_PATH".to_string(),
        payload.base.transcript_path.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_HOOK_EVENT".to_string(),
        "SubagentStart".to_string(),
    );
    env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
    env_vars.insert(
        "CONCLAUDE_CONFIG_DIR".to_string(),
        config_dir.to_string_lossy().to_string(),
    );

    // Full JSON payload for advanced use cases
    if let Ok(json) = serde_json::to_string(payload) {
        env_vars.insert("CONCLAUDE_PAYLOAD_JSON".to_string(), json);
    }

    env_vars
}

/// Collect skill start commands from configuration for matching patterns
///
/// # Errors
///
/// Returns an error if bash command extraction fails.
pub(crate) fn collect_skill_start_commands(
    config: &SkillStartConfig,
    matching_patterns: &[&str],
) -> Result<Vec<SkillStartCommandConfig>> {
    let mut commands = Vec::new();

    for pattern in matching_patterns {
        if let Some(cmd_list) = config.commands.get(*pattern) {
            for cmd_config in cmd_list {
                let extracted = extract_bash_commands(&cmd_config.run)?;
                let show_stdout = cmd_config.show_stdout.unwrap_or(false);
                let show_stderr = cmd_config.show_stderr.unwrap_or(false);
                let show_command = cmd_config.show_command.unwrap_or(true);
                let max_output_lines = cmd_config.max_output_lines;
                let notify_per_command = cmd_config.notify_per_command.unwrap_or(false);

                for cmd in extracted {
                    commands.push(SkillStartCommandConfig {
                        command: cmd,
                        show_stdout,
                        show_stderr,
                        max_output_lines,
                        timeout: cmd_config.timeout,
                        show_command,
                        notify_per_command,
                    });
                }
            }
        }
    }

    Ok(commands)
}

/// Execute skill start hook commands with environment variables
///
/// # Errors
///
/// Returns an error if command spawning fails. Individual command failures are logged
/// but do not stop subsequent command execution.
async fn execute_skill_start_commands(
    commands: &[SkillStartCommandConfig],
    env_vars: &HashMap<String, String>,
    config_dir: &Path,
) -> Result<()> {
    if commands.is_empty() {
        return Ok(());
    }

    println!("Executing {} skill start hook commands", commands.len());

    for (index, cmd_config) in commands.iter().enumerate() {
        if cmd_config.show_command {
            println!(
                "Executing skill start command {}/{}: {}",
                index + 1,
                commands.len(),
                cmd_config.command
            );
        } else {
            println!(
                "Executing skill start command {}/{}",
                index + 1,
                commands.len()
            );
        }

        // Send start notification if per-command notifications are enabled
        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Running: {}", cmd_config.command)
            } else {
                "Running command".to_string()
            };
            send_notification("SubagentStart", "running", Some(&context_msg));
        }

        let child = TokioCommand::new("bash")
            .arg("-c")
            .arg(&cmd_config.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env_vars)
            .current_dir(config_dir)
            .spawn();

        let child = match child {
            Ok(c) => c,
            Err(e) => {
                // Log error but continue to next command
                if cmd_config.show_command {
                    eprintln!(
                        "Failed to spawn skill start command '{}': {}",
                        cmd_config.command, e
                    );
                } else {
                    eprintln!("Failed to spawn skill start command: {}", e);
                }

                if cmd_config.notify_per_command {
                    let context_msg = if cmd_config.show_command {
                        format!("Failed to spawn command: {}", cmd_config.command)
                    } else {
                        "Failed to spawn command".to_string()
                    };
                    send_notification("SubagentStart", "failure", Some(&context_msg));
                }

                continue;
            }
        };

        let output = if let Some(timeout_secs) = cmd_config.timeout {
            match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(result) => match result {
                    Ok(o) => o,
                    Err(e) => {
                        if cmd_config.show_command {
                            eprintln!(
                                "Failed to wait for skill start command '{}': {}",
                                cmd_config.command, e
                            );
                        } else {
                            eprintln!("Failed to wait for skill start command: {}", e);
                        }

                        if cmd_config.notify_per_command {
                            let context_msg = if cmd_config.show_command {
                                format!("Command failed to wait: {}", cmd_config.command)
                            } else {
                                "Command failed to wait".to_string()
                            };
                            send_notification("SubagentStart", "failure", Some(&context_msg));
                        }

                        continue;
                    }
                },
                Err(_) => {
                    if cmd_config.show_command {
                        eprintln!(
                            "Skill start command timed out after {} seconds: {}",
                            timeout_secs, cmd_config.command
                        );
                    } else {
                        eprintln!(
                            "Skill start command timed out after {} seconds",
                            timeout_secs
                        );
                    }

                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command timed out: {}", cmd_config.command)
                        } else {
                            "Command timed out".to_string()
                        };
                        send_notification("SubagentStart", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        } else {
            match child.wait_with_output().await {
                Ok(o) => o,
                Err(e) => {
                    if cmd_config.show_command {
                        eprintln!(
                            "Failed to wait for skill start command '{}': {}",
                            cmd_config.command, e
                        );
                    } else {
                        eprintln!("Failed to wait for skill start command: {}", e);
                    }

                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command failed to wait: {}", cmd_config.command)
                        } else {
                            "Command failed to wait".to_string()
                        };
                        send_notification("SubagentStart", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(1);

            if cmd_config.show_command {
                eprintln!(
                    "Skill start command failed (exit code: {}): {}",
                    exit_code, cmd_config.command
                );
            } else {
                eprintln!("Skill start command failed (exit code: {})", exit_code);
            }

            if cmd_config.show_stdout && !stdout.trim().is_empty() {
                eprintln!("Stdout: {}", stdout.trim());
            }
            if cmd_config.show_stderr && !stderr.trim().is_empty() {
                eprintln!("Stderr: {}", stderr.trim());
            }

            if cmd_config.notify_per_command {
                let context_msg = if cmd_config.show_command {
                    format!(
                        "Command failed (exit code: {}): {}",
                        exit_code, cmd_config.command
                    )
                } else {
                    format!("Command failed (exit code: {})", exit_code)
                };
                send_notification("SubagentStart", "failure", Some(&context_msg));
            }

            continue;
        }

        // Successful command - show output if configured
        if cmd_config.show_stdout && !stdout.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stdout.to_string()
            };
            println!("Stdout: {}", output_to_show);
        }

        if cmd_config.show_stderr && !stderr.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stderr.to_string()
            };
            eprintln!("Stderr: {}", output_to_show);
        }

        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Command completed: {}", cmd_config.command)
            } else {
                "Command completed".to_string()
            };
            send_notification("SubagentStart", "success", Some(&context_msg));
        }
    }

    println!("All skill start hook commands completed");
    Ok(())
}

/// Handles `SubagentStop` hook events when Claude subagents complete their tasks.
///
/// This function processes subagent stop events by:
/// 1. Validating the payload (requires agent_id and agent_transcript_path)
/// 2. Loading configuration to check for subagentStop commands
/// 3. Matching agent_id against configured patterns
/// 4. Executing matching commands with environment variables
/// 5. Sending notifications
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
pub async fn handle_subagent_stop() -> Result<HookResult> {
    let payload: SubagentStopPayload = read_payload_from_stdin()?;

    // Validate the payload including agent_id and agent_transcript_path fields
    validate_subagent_stop_payload(&payload).map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "Processing SubagentStop hook: session_id={}, agent_id={}",
        payload.base.session_id, payload.agent_id
    );

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Set environment variables for the subagent's information
    std::env::set_var("CONCLAUDE_AGENT_ID", &payload.agent_id);
    std::env::set_var(
        "CONCLAUDE_AGENT_TRANSCRIPT_PATH",
        &payload.agent_transcript_path,
    );

    // Set CONCLAUDE_AGENT_NAME to the agent name from env var, or fall back to agent_id
    let agent_name_value = agent_name.as_deref().unwrap_or(&payload.agent_id);
    std::env::set_var("CONCLAUDE_AGENT_NAME", agent_name_value);

    if let Some(name) = &agent_name {
        println!(
            "Using agent name '{}' for agent_id '{}'",
            name, payload.agent_id
        );
    } else {
        println!(
            "No agent name provided, using agent_id '{}' as fallback",
            payload.agent_id
        );
    }

    // Load configuration
    let (config, config_path) = get_config().await?;
    let config_dir = get_config_dir(config_path);

    // Check if subagentStop commands are configured
    if !config.subagent_stop.commands.is_empty() {
        // Match agent_id against configured patterns
        let matching_patterns = match_subagent_patterns(&payload.agent_id, &config.subagent_stop)?;

        if !matching_patterns.is_empty() {
            println!(
                "Agent '{}' matched patterns: {:?}",
                payload.agent_id, matching_patterns
            );

            // Collect commands for matching patterns
            let commands =
                collect_subagent_stop_commands(&config.subagent_stop, &matching_patterns)?;

            if !commands.is_empty() {
                // Build environment variables
                let env_vars = build_subagent_env_vars(&payload, config_dir, agent_name.as_deref());

                // Execute commands (graceful failure handling)
                execute_subagent_stop_commands(&commands, &env_vars, config_dir).await?;
            }
        } else {
            println!(
                "Agent '{}' did not match any configured patterns",
                payload.agent_id
            );
        }
    }

    // Send notification for subagent stop with agent ID included
    send_notification(
        "SubagentStop",
        "success",
        Some(&format!("Subagent '{}' completed", payload.agent_id)),
    );

    Ok(HookResult::success())
}

/// Match slash command against configured patterns in SlashCommandConfig
///
/// Returns a vector of matched pattern strings, with wildcard "*" first (if matched),
/// followed by other matching patterns in stable order.
pub(crate) fn match_slash_command_patterns<'a>(
    command: &str,
    config: &'a SlashCommandConfig,
) -> Result<Vec<&'a str>> {
    let mut wildcard_matches = Vec::new();
    let mut other_matches = Vec::new();

    for pattern_str in config.commands.keys() {
        // Handle wildcard pattern specially - it always matches
        if pattern_str == "*" {
            wildcard_matches.push(pattern_str.as_str());
            continue;
        }

        let pattern_to_match = pattern_str.strip_prefix('/').unwrap_or(pattern_str);

        // Use glob pattern matching for other patterns
        let pattern = Pattern::new(pattern_to_match).with_context(|| {
            format!(
                "Invalid glob pattern in slashCommands config: {}",
                pattern_str
            )
        })?;

        if pattern.matches(command) {
            other_matches.push(pattern_str.as_str());
        }
    }

    // Sort non-wildcard matches for consistent ordering
    other_matches.sort();

    // Wildcard first, then sorted other matches
    let mut result = wildcard_matches;
    result.extend(other_matches);
    Ok(result)
}

/// Build environment variables for slash command hook execution
#[must_use]
pub(crate) fn build_slash_command_env_vars(
    payload: &UserPromptSubmitPayload,
    config_dir: &Path,
    detection: &SlashCommandDetection,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    // Slash command-specific environment variables
    env_vars.insert(
        "CONCLAUDE_SLASH_COMMAND".to_string(),
        detection.command.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_SLASH_COMMAND_ARGS".to_string(),
        detection.args.clone(),
    );

    // User prompt environment variable
    if let Some(ref prompt) = payload.prompt {
        env_vars.insert("CONCLAUDE_USER_PROMPT".to_string(), prompt.clone());
    }

    // Session-level environment variables
    env_vars.insert(
        "CONCLAUDE_SESSION_ID".to_string(),
        payload.base.session_id.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_TRANSCRIPT_PATH".to_string(),
        payload.base.transcript_path.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_HOOK_EVENT".to_string(),
        "UserPromptSubmit".to_string(),
    );
    env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
    env_vars.insert(
        "CONCLAUDE_CONFIG_DIR".to_string(),
        config_dir.to_string_lossy().to_string(),
    );

    env_vars
}

/// Collect slash command entries from configuration for matching patterns
pub(crate) fn collect_slash_command_entries(
    config: &SlashCommandConfig,
    matching_patterns: &[&str],
) -> Result<Vec<SlashCommandEntryConfig>> {
    let mut commands = Vec::new();

    for pattern in matching_patterns {
        if let Some(cmd_list) = config.commands.get(*pattern) {
            for cmd_config in cmd_list {
                let extracted = extract_bash_commands(&cmd_config.run)?;
                let show_stdout = cmd_config.show_stdout.unwrap_or(false);
                let show_stderr = cmd_config.show_stderr.unwrap_or(false);
                let show_command = cmd_config.show_command.unwrap_or(true);
                let max_output_lines = cmd_config.max_output_lines;
                let notify_per_command = cmd_config.notify_per_command.unwrap_or(false);

                for cmd in extracted {
                    commands.push(SlashCommandEntryConfig {
                        command: cmd,
                        show_stdout,
                        show_stderr,
                        max_output_lines,
                        timeout: cmd_config.timeout,
                        show_command,
                        notify_per_command,
                    });
                }
            }
        }
    }

    Ok(commands)
}

/// Execute slash command hooks with environment variables
///
/// Returns Ok(Some(message)) if a command blocked (exit code 2),
/// Ok(None) if all commands succeeded, or Err on execution failure.
async fn execute_slash_command_hooks(
    commands: &[SlashCommandEntryConfig],
    env_vars: &HashMap<String, String>,
    config_dir: &Path,
) -> Result<Option<String>> {
    if commands.is_empty() {
        return Ok(None);
    }

    println!("Executing {} slash command hook(s)", commands.len());

    for (index, cmd_config) in commands.iter().enumerate() {
        if cmd_config.show_command {
            println!(
                "Executing slash command hook {}/{}: {}",
                index + 1,
                commands.len(),
                cmd_config.command
            );
        } else {
            println!(
                "Executing slash command hook {}/{}",
                index + 1,
                commands.len()
            );
        }

        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Running: {}", cmd_config.command)
            } else {
                "Running command".to_string()
            };
            send_notification("UserPromptSubmit", "running", Some(&context_msg));
        }

        let child = TokioCommand::new("bash")
            .arg("-c")
            .arg(&cmd_config.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env_vars)
            .current_dir(config_dir)
            .spawn();

        let child = match child {
            Ok(c) => c,
            Err(e) => {
                if cmd_config.show_command {
                    eprintln!(
                        "Failed to spawn slash command hook '{}': {}",
                        cmd_config.command, e
                    );
                } else {
                    eprintln!("Failed to spawn slash command hook: {}", e);
                }

                if cmd_config.notify_per_command {
                    let context_msg = if cmd_config.show_command {
                        format!("Failed to spawn: {}", cmd_config.command)
                    } else {
                        "Failed to spawn command".to_string()
                    };
                    send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                }

                continue;
            }
        };

        let output = if let Some(timeout_secs) = cmd_config.timeout {
            match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(result) => match result {
                    Ok(o) => o,
                    Err(e) => {
                        if cmd_config.show_command {
                            eprintln!(
                                "Failed to wait for slash command hook '{}': {}",
                                cmd_config.command, e
                            );
                        } else {
                            eprintln!("Failed to wait for slash command hook: {}", e);
                        }

                        if cmd_config.notify_per_command {
                            let context_msg = if cmd_config.show_command {
                                format!("Failed to wait: {}", cmd_config.command)
                            } else {
                                "Failed to wait for command".to_string()
                            };
                            send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                        }

                        continue;
                    }
                },
                Err(_) => {
                    if cmd_config.show_command {
                        eprintln!(
                            "Slash command hook timed out after {} seconds: {}",
                            timeout_secs, cmd_config.command
                        );
                    } else {
                        eprintln!(
                            "Slash command hook timed out after {} seconds",
                            timeout_secs
                        );
                    }

                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Command timed out: {}", cmd_config.command)
                        } else {
                            "Command timed out".to_string()
                        };
                        send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        } else {
            match child.wait_with_output().await {
                Ok(o) => o,
                Err(e) => {
                    if cmd_config.show_command {
                        eprintln!(
                            "Failed to wait for slash command hook '{}': {}",
                            cmd_config.command, e
                        );
                    } else {
                        eprintln!("Failed to wait for slash command hook: {}", e);
                    }

                    if cmd_config.notify_per_command {
                        let context_msg = if cmd_config.show_command {
                            format!("Failed to wait: {}", cmd_config.command)
                        } else {
                            "Failed to wait for command".to_string()
                        };
                        send_notification("UserPromptSubmit", "failure", Some(&context_msg));
                    }

                    continue;
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let exit_code = output.status.code().unwrap_or(1);

        // Exit code 2 means block the operation
        if exit_code == 2 {
            let message = if cmd_config.show_command {
                format!(
                    "Slash command blocked by hook (exit code: {}): {}",
                    exit_code, cmd_config.command
                )
            } else {
                format!("Slash command blocked by hook (exit code: {})", exit_code)
            };

            if cmd_config.show_stdout && !stdout.trim().is_empty() {
                eprintln!("Stdout: {}", stdout.trim());
            }
            if cmd_config.show_stderr && !stderr.trim().is_empty() {
                eprintln!("Stderr: {}", stderr.trim());
            }

            if cmd_config.notify_per_command {
                let context_msg = if cmd_config.show_command {
                    format!("Command blocked: {}", cmd_config.command)
                } else {
                    "Command blocked".to_string()
                };
                send_notification("UserPromptSubmit", "blocked", Some(&context_msg));
            }

            return Ok(Some(message));
        }

        if !output.status.success() {
            if cmd_config.show_command {
                eprintln!(
                    "Slash command hook failed (exit code: {}): {}",
                    exit_code, cmd_config.command
                );
            } else {
                eprintln!("Slash command hook failed (exit code: {})", exit_code);
            }

            if cmd_config.show_stdout && !stdout.trim().is_empty() {
                eprintln!("Stdout: {}", stdout.trim());
            }
            if cmd_config.show_stderr && !stderr.trim().is_empty() {
                eprintln!("Stderr: {}", stderr.trim());
            }

            if cmd_config.notify_per_command {
                let context_msg = if cmd_config.show_command {
                    format!(
                        "Command failed (exit code: {}): {}",
                        exit_code, cmd_config.command
                    )
                } else {
                    format!("Command failed (exit code: {})", exit_code)
                };
                send_notification("UserPromptSubmit", "failure", Some(&context_msg));
            }

            continue;
        }

        // Successful command - show output if configured
        if cmd_config.show_stdout && !stdout.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stdout, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stdout.to_string()
            };
            println!("Stdout: {}", output_to_show);
        }

        if cmd_config.show_stderr && !stderr.trim().is_empty() {
            let output_to_show = if let Some(max_lines) = cmd_config.max_output_lines {
                let (truncated, is_truncated, omitted) = truncate_output(&stderr, max_lines);
                if is_truncated {
                    format!("{}\n... ({} lines omitted)", truncated, omitted)
                } else {
                    truncated
                }
            } else {
                stderr.to_string()
            };
            eprintln!("Stderr: {}", output_to_show);
        }

        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Command completed: {}", cmd_config.command)
            } else {
                "Command completed".to_string()
            };
            send_notification("UserPromptSubmit", "success", Some(&context_msg));
        }
    }

    println!("All slash command hooks completed");
    Ok(None)
}

/// Handles `PreCompact` hook events before transcript compaction occurs.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
#[allow(clippy::unused_async)]
pub async fn handle_pre_compact() -> Result<HookResult> {
    let payload: PreCompactPayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    // Read agent name from environment variable (set by CLI --agent flag)
    let agent_name = std::env::var(AGENT_ENV_VAR).ok();

    // Export CONCLAUDE_AGENT_NAME for any commands that are executed
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    println!(
        "Processing PreCompact hook: session_id={}, trigger={:?}",
        payload.base.session_id, payload.trigger
    );

    // Send notification for pre-compact hook
    send_notification(
        "PreCompact",
        "success",
        Some(&format!("Compaction triggered: {:?}", payload.trigger)),
    );
    Ok(HookResult::success())
}

/// Check tool usage validation rules
///
/// # Errors
///
/// Returns an error if configuration loading fails or glob pattern creation fails.
async fn check_tool_usage_rules(payload: &PreToolUsePayload) -> Result<Option<HookResult>> {
    let (config, _config_path) = get_config().await?;

    // Detect current agent context from environment variable (set by CLI --agent flag)
    let current_agent = std::env::var(AGENT_ENV_VAR).unwrap_or_else(|_| "main".to_string());

    for rule in &config.pre_tool_use.tool_usage_validation {
        if rule.tool == payload.tool_name || rule.tool == "*" {
            // Check agent match - skip rule if it doesn't apply to current agent
            let agent_pattern = rule.agent.as_deref().unwrap_or("*");
            if !matches_agent_pattern(&current_agent, agent_pattern) {
                continue;
            }
            // Check if this is a Bash command with a commandPattern rule
            if payload.tool_name == "Bash" && rule.command_pattern.is_some() {
                // Extract the command
                if let Some(command) = extract_bash_command(&payload.tool_input) {
                    let pattern = rule.command_pattern.as_ref().unwrap();
                    let mode = rule.match_mode.as_deref().unwrap_or("full");

                    // Perform pattern matching based on mode
                    let matches = if mode == "prefix" {
                        // Prefix mode: test progressively longer prefixes
                        let glob = Pattern::new(pattern)?;
                        let words: Vec<&str> = command.split_whitespace().collect();
                        (1..=words.len()).any(|i| {
                            let prefix = words[..i].join(" ");
                            glob.matches(&prefix)
                        })
                    } else {
                        // Full mode: match entire command
                        Pattern::new(pattern)?.matches(&command)
                    };

                    // Handle actions based on match result
                    if rule.action == "block" && matches {
                        let message = rule.message.clone().unwrap_or_else(|| {
                            format!(
                                "Bash command blocked by preToolUse.toolUsageValidation rule: {}",
                                pattern
                            )
                        });
                        return Ok(Some(HookResult::blocked(message)));
                    } else if rule.action == "allow" && !matches {
                        let message = rule.message.clone().unwrap_or_else(|| {
                            format!(
                                "Bash command blocked: does not match preToolUse.toolUsageValidation allow rule pattern: {}",
                                pattern
                            )
                        });
                        return Ok(Some(HookResult::blocked(message)));
                    } else if rule.action == "allow" && matches {
                        // Allow and stop checking further rules for this command
                        return Ok(None);
                    }
                }
                // Skip file-path validation for Bash command rules
                continue;
            }

            // Extract file path if available
            if let Some(file_path) = extract_file_path(&payload.tool_input) {
                let matches = Pattern::new(&rule.pattern)?.matches(&file_path);

                if (rule.action == "block" && matches) || (rule.action == "allow" && !matches) {
                    let message = rule.message.clone().unwrap_or_else(|| {
                        format!(
                            "Tool usage blocked by preToolUse.toolUsageValidation rule: {}",
                            rule.pattern
                        )
                    });
                    return Ok(Some(HookResult::blocked(message)));
                }
            }
        }
    }

    Ok(None)
}

/// Check if a file is git-ignored and should be protected.
///
/// This check blocks both creation of new files and modification of existing files
/// that match `.gitignore` patterns. This is intentional - if a file should be
/// git-ignored, Claude shouldn't create or modify it.
///
/// Note: Currently only loads `.gitignore` from the repository root. Nested
/// `.gitignore` files in subdirectories are not supported.
///
/// # Errors
///
/// Returns an error if configuration loading fails or gitignore check fails.
async fn check_git_ignored_file(payload: &PreToolUsePayload) -> Result<Option<HookResult>> {
    let (config, config_path) = get_config().await?;

    // Only check if the feature is enabled
    if !config.pre_tool_use.prevent_update_git_ignored {
        return Ok(None);
    }

    // Extract file path from tool input
    let file_path = extract_file_path(&payload.tool_input);
    let Some(file_path) = file_path else {
        return Ok(None);
    };

    // Find the actual git repository root by walking up from config path
    // This is more reliable than just using config path's parent
    let config_dir = get_config_dir(config_path);
    let repo_root = match find_git_root(config_dir) {
        Some(root) => root,
        None => {
            // Not in a git repository - skip this check
            return Ok(None);
        }
    };

    // Resolve the file path to check
    let cwd = std::env::current_dir().context("Failed to get current working directory")?;
    let resolved_path = cwd.join(&file_path);

    // Check if the file is git-ignored
    let (is_ignored, pattern) = is_path_git_ignored(&resolved_path, &repo_root)?;

    if is_ignored {
        let pattern_display =
            pattern.unwrap_or_else(|| format!("(pattern in {}/.gitignore)", repo_root.display()));

        let message = format!(
            "File operation blocked: Path is git-ignored\n\
            \n\
            File: {}\n\
            Matched pattern in .gitignore: {}\n\
            \n\
            This file is protected by 'preventUpdateGitIgnored: true'\n\
            \n\
            To allow modifications:\n\
            1. Remove the pattern from .gitignore\n\
            2. Use a negation pattern (e.g., !{})\n\
            3. Set preventUpdateGitIgnored: false in your config",
            file_path,
            pattern_display,
            Path::new(&file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file_path)
        );

        eprintln!(
            "PreToolUse blocked git-ignored file: tool_name={}, file_path={}, pattern={}",
            payload.tool_name, file_path, pattern_display
        );

        return Ok(Some(HookResult::blocked(message)));
    }

    Ok(None)
}

/// Snapshot the root directory
///
/// # Errors
///
/// Returns an error if the current directory cannot be read.
fn snapshot_root_directory() -> Result<HashSet<String>> {
    let mut snapshot = HashSet::new();

    for entry in (fs::read_dir(".")?).flatten() {
        if let Ok(file_name) = entry.file_name().into_string() {
            snapshot.insert(file_name);
        }
    }

    Ok(snapshot)
}

/// Check for new additions to the root directory
///
/// # Errors
///
/// Returns an error if the current directory cannot be read.
fn check_root_additions(snapshot: &HashSet<String>) -> Result<Option<HookResult>> {
    let mut new_files = Vec::new();

    for entry in (fs::read_dir(".")?).flatten() {
        if let Ok(file_name) = entry.file_name().into_string() {
            if !snapshot.contains(&file_name) && !file_name.starts_with('.') {
                new_files.push(file_name);
            }
        }
    }

    if !new_files.is_empty() {
        let message = format!(
            "Unauthorized root additions detected: {}",
            new_files.join(", ")
        );
        return Ok(Some(HookResult::blocked(message)));
    }

    Ok(None)
}

/// Generic command config used by the pattern-based hook command execution pipeline
pub(crate) struct GenericCommandConfig {
    pub command: String,
    pub message: Option<String>,
    pub show_stdout: bool,
    pub show_stderr: bool,
    pub max_output_lines: Option<u32>,
    pub timeout: Option<u64>,
    pub show_command: bool,
    pub notify_per_command: bool,
}

/// Generic pattern matching function that works with any HashMap<String, Vec<T>> config.
/// Returns matched pattern strings, with wildcard "*" first.
pub(crate) fn match_generic_patterns<'a, T>(
    value: &str,
    commands: &'a HashMap<String, Vec<T>>,
) -> Result<Vec<&'a str>> {
    let mut wildcard_matches = Vec::new();
    let mut other_matches = Vec::new();

    for pattern_str in commands.keys() {
        if pattern_str == "*" {
            wildcard_matches.push(pattern_str.as_str());
            continue;
        }

        let pattern = Pattern::new(pattern_str).with_context(|| {
            format!("Invalid glob pattern in config: {}", pattern_str)
        })?;

        if pattern.matches(value) {
            other_matches.push(pattern_str.as_str());
        }
    }

    other_matches.sort();

    let mut result = wildcard_matches;
    result.extend(other_matches);
    Ok(result)
}

/// Collect commands from TeammateIdleConfig for matching patterns
fn collect_teammate_idle_commands(
    config: &TeammateIdleConfig,
    matching_patterns: &[&str],
) -> Result<Vec<GenericCommandConfig>> {
    let mut commands = Vec::new();
    for pattern in matching_patterns {
        if let Some(cmd_list) = config.commands.get(*pattern) {
            for cmd_config in cmd_list {
                let extracted = extract_bash_commands(&cmd_config.run)?;
                for cmd in extracted {
                    commands.push(GenericCommandConfig {
                        command: cmd,
                        message: cmd_config.message.clone(),
                        show_stdout: cmd_config.show_stdout.unwrap_or(false),
                        show_stderr: cmd_config.show_stderr.unwrap_or(false),
                        max_output_lines: cmd_config.max_output_lines,
                        timeout: cmd_config.timeout,
                        show_command: cmd_config.show_command.unwrap_or(true),
                        notify_per_command: cmd_config.notify_per_command.unwrap_or(false),
                    });
                }
            }
        }
    }
    Ok(commands)
}

/// Collect commands from TaskCompletedConfig for matching patterns
fn collect_task_completed_commands(
    config: &TaskCompletedConfig,
    matching_patterns: &[&str],
) -> Result<Vec<GenericCommandConfig>> {
    let mut commands = Vec::new();
    for pattern in matching_patterns {
        if let Some(cmd_list) = config.commands.get(*pattern) {
            for cmd_config in cmd_list {
                let extracted = extract_bash_commands(&cmd_config.run)?;
                for cmd in extracted {
                    commands.push(GenericCommandConfig {
                        command: cmd,
                        message: cmd_config.message.clone(),
                        show_stdout: cmd_config.show_stdout.unwrap_or(false),
                        show_stderr: cmd_config.show_stderr.unwrap_or(false),
                        max_output_lines: cmd_config.max_output_lines,
                        timeout: cmd_config.timeout,
                        show_command: cmd_config.show_command.unwrap_or(true),
                        notify_per_command: cmd_config.notify_per_command.unwrap_or(false),
                    });
                }
            }
        }
    }
    Ok(commands)
}

/// Collect commands from ConfigChangeConfig for matching patterns
fn collect_config_change_commands(
    config: &ConfigChangeConfig,
    matching_patterns: &[&str],
) -> Result<Vec<GenericCommandConfig>> {
    let mut commands = Vec::new();
    for pattern in matching_patterns {
        if let Some(cmd_list) = config.commands.get(*pattern) {
            for cmd_config in cmd_list {
                let extracted = extract_bash_commands(&cmd_config.run)?;
                for cmd in extracted {
                    commands.push(GenericCommandConfig {
                        command: cmd,
                        message: cmd_config.message.clone(),
                        show_stdout: cmd_config.show_stdout.unwrap_or(false),
                        show_stderr: cmd_config.show_stderr.unwrap_or(false),
                        max_output_lines: cmd_config.max_output_lines,
                        timeout: cmd_config.timeout,
                        show_command: cmd_config.show_command.unwrap_or(true),
                        notify_per_command: cmd_config.notify_per_command.unwrap_or(false),
                    });
                }
            }
        }
    }
    Ok(commands)
}

/// Build environment variables for TeammateIdle hook execution
fn build_teammate_idle_env_vars(
    payload: &TeammateIdlePayload,
    config_dir: &Path,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    env_vars.insert(
        "CONCLAUDE_TEAMMATE_NAME".to_string(),
        payload.teammate_name.clone(),
    );
    env_vars.insert("CONCLAUDE_TEAM_NAME".to_string(), payload.team_name.clone());
    env_vars.insert(
        "CONCLAUDE_SESSION_ID".to_string(),
        payload.base.session_id.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_TRANSCRIPT_PATH".to_string(),
        payload.base.transcript_path.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_HOOK_EVENT".to_string(),
        "TeammateIdle".to_string(),
    );
    env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
    env_vars.insert(
        "CONCLAUDE_CONFIG_DIR".to_string(),
        config_dir.to_string_lossy().to_string(),
    );
    if let Ok(json) = serde_json::to_string(payload) {
        env_vars.insert("CONCLAUDE_PAYLOAD_JSON".to_string(), json);
    }
    env_vars
}

/// Build environment variables for TaskCompleted hook execution
fn build_task_completed_env_vars(
    payload: &TaskCompletedPayload,
    config_dir: &Path,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    env_vars.insert("CONCLAUDE_TASK_ID".to_string(), payload.task_id.clone());
    env_vars.insert(
        "CONCLAUDE_TASK_SUBJECT".to_string(),
        payload.task_subject.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_TASK_DESCRIPTION".to_string(),
        payload.task_description.clone().unwrap_or_default(),
    );
    env_vars.insert(
        "CONCLAUDE_SESSION_ID".to_string(),
        payload.base.session_id.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_TRANSCRIPT_PATH".to_string(),
        payload.base.transcript_path.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_HOOK_EVENT".to_string(),
        "TaskCompleted".to_string(),
    );
    env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
    env_vars.insert(
        "CONCLAUDE_CONFIG_DIR".to_string(),
        config_dir.to_string_lossy().to_string(),
    );
    if let Ok(json) = serde_json::to_string(payload) {
        env_vars.insert("CONCLAUDE_PAYLOAD_JSON".to_string(), json);
    }
    env_vars
}

/// Build environment variables for ConfigChange hook execution
fn build_config_change_env_vars(
    payload: &ConfigChangePayload,
    config_dir: &Path,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    env_vars.insert(
        "CONCLAUDE_CONFIG_SOURCE".to_string(),
        payload.source.to_string(),
    );
    env_vars.insert(
        "CONCLAUDE_CONFIG_FILE_PATH".to_string(),
        payload.file_path.clone().unwrap_or_default(),
    );
    env_vars.insert(
        "CONCLAUDE_SESSION_ID".to_string(),
        payload.base.session_id.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_TRANSCRIPT_PATH".to_string(),
        payload.base.transcript_path.clone(),
    );
    env_vars.insert(
        "CONCLAUDE_HOOK_EVENT".to_string(),
        "ConfigChange".to_string(),
    );
    env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
    env_vars.insert(
        "CONCLAUDE_CONFIG_DIR".to_string(),
        config_dir.to_string_lossy().to_string(),
    );
    if let Ok(json) = serde_json::to_string(payload) {
        env_vars.insert("CONCLAUDE_PAYLOAD_JSON".to_string(), json);
    }
    env_vars
}

/// Execute commands for pattern-based hooks. Returns Some(blocked) if exit code 2.
async fn execute_generic_commands(
    commands: &[GenericCommandConfig],
    env_vars: &HashMap<String, String>,
    config_dir: &Path,
    hook_name: &str,
) -> Result<Option<HookResult>> {
    if commands.is_empty() {
        return Ok(None);
    }

    println!("Executing {} {} hook commands", commands.len(), hook_name);

    for (index, cmd_config) in commands.iter().enumerate() {
        if cmd_config.show_command {
            println!(
                "Executing {} command {}/{}: {}",
                hook_name,
                index + 1,
                commands.len(),
                cmd_config.command
            );
        } else {
            println!(
                "Executing {} command {}/{}",
                hook_name,
                index + 1,
                commands.len()
            );
        }

        if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Running: {}", cmd_config.command)
            } else {
                "Running command".to_string()
            };
            send_notification(hook_name, "running", Some(&context_msg));
        }

        let child = TokioCommand::new("bash")
            .arg("-c")
            .arg(&cmd_config.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env_vars)
            .current_dir(config_dir)
            .spawn();

        let child = match child {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to spawn {} command: {}", hook_name, e);
                continue;
            }
        };

        let output = if let Some(timeout_secs) = cmd_config.timeout {
            match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(Ok(o)) => o,
                Ok(Err(e)) => {
                    eprintln!("Failed to wait for {} command: {}", hook_name, e);
                    continue;
                }
                Err(_) => {
                    eprintln!(
                        "{} command timed out after {} seconds",
                        hook_name, timeout_secs
                    );
                    continue;
                }
            }
        } else {
            match child.wait_with_output().await {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("Failed to wait for {} command: {}", hook_name, e);
                    continue;
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Show output if configured
        if cmd_config.show_stdout && !stdout.is_empty() {
            let (truncated, is_truncated, omitted) =
                if let Some(max_lines) = cmd_config.max_output_lines {
                    truncate_output(&stdout, max_lines)
                } else {
                    (stdout.to_string(), false, 0)
                };
            println!("[stdout] {}", truncated);
            if is_truncated {
                println!("... ({} lines omitted)", omitted);
            }
        }

        if cmd_config.show_stderr && !stderr.is_empty() {
            let (truncated, is_truncated, omitted) =
                if let Some(max_lines) = cmd_config.max_output_lines {
                    truncate_output(&stderr, max_lines)
                } else {
                    (stderr.to_string(), false, 0)
                };
            eprintln!("[stderr] {}", truncated);
            if is_truncated {
                eprintln!("... ({} lines omitted)", omitted);
            }
        }

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(1);

            // Exit code 2 means "block this operation"
            if exit_code == 2 {
                let block_msg = cmd_config.message.clone().unwrap_or_else(|| {
                    format!("{} hook blocked by command", hook_name)
                });
                println!("{} hook BLOCKED: {}", hook_name, block_msg);

                if cmd_config.notify_per_command {
                    send_notification(hook_name, "blocked", Some(&block_msg));
                }

                return Ok(Some(HookResult::blocked(block_msg)));
            }

            // Other non-zero exits are logged but don't block
            if let Some(ref custom_msg) = cmd_config.message {
                eprintln!(
                    "{} command failed (exit code {}): {}",
                    hook_name, exit_code, custom_msg
                );
            } else {
                eprintln!("{} command failed (exit code {})", hook_name, exit_code);
            }
        } else if cmd_config.notify_per_command {
            let context_msg = if cmd_config.show_command {
                format!("Completed: {}", cmd_config.command)
            } else {
                "Command completed".to_string()
            };
            send_notification(hook_name, "success", Some(&context_msg));
        }
    }

    Ok(None)
}

/// Handles `PostToolUseFailure` hook events when a tool execution fails.
/// This is an observational hook - it cannot block operations.
///
/// # Errors
///
/// Returns an error if payload reading or validation fails.
#[allow(clippy::unused_async)]
pub async fn handle_post_tool_use_failure() -> Result<HookResult> {
    let payload: PostToolUseFailurePayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    if payload.tool_name.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: tool_name"));
    }
    if payload.error.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: error"));
    }

    let agent_name = std::env::var(AGENT_ENV_VAR).ok();
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    // Set tool-specific environment variables
    std::env::set_var("CONCLAUDE_TOOL_NAME", &payload.tool_name);
    std::env::set_var("CONCLAUDE_TOOL_ERROR", &payload.error);

    println!(
        "Processing PostToolUseFailure hook: session_id={}, tool_name={}, error={}",
        payload.base.session_id, payload.tool_name, payload.error
    );

    send_notification(
        "PostToolUseFailure",
        "failure",
        Some(&format!(
            "Tool '{}' failed: {}",
            payload.tool_name, payload.error
        )),
    );

    Ok(HookResult::success())
}

/// Handles `WorktreeRemove` hook events when a git worktree is being removed.
/// This is an observational hook for cleanup - it cannot block operations.
///
/// # Errors
///
/// Returns an error if payload reading or validation fails.
#[allow(clippy::unused_async)]
pub async fn handle_worktree_remove() -> Result<HookResult> {
    let payload: WorktreeRemovePayload = read_payload_from_stdin()?;

    validate_worktree_remove_payload(&payload).map_err(|e| anyhow::anyhow!(e))?;

    let agent_name = std::env::var(AGENT_ENV_VAR).ok();
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    std::env::set_var("CONCLAUDE_WORKTREE_PATH", &payload.worktree_path);

    println!(
        "Processing WorktreeRemove hook: session_id={}, worktree_path={}",
        payload.base.session_id, payload.worktree_path
    );

    send_notification(
        "WorktreeRemove",
        "success",
        Some(&format!("Worktree removed: {}", payload.worktree_path)),
    );

    Ok(HookResult::success())
}

/// Handles `TeammateIdle` hook events when a teammate agent becomes idle.
/// Can block the idle if configured commands exit with code 2.
///
/// # Errors
///
/// Returns an error if payload reading, validation, or command execution fails.
pub async fn handle_teammate_idle() -> Result<HookResult> {
    let payload: TeammateIdlePayload = read_payload_from_stdin()?;

    validate_teammate_idle_payload(&payload).map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "Processing TeammateIdle hook: session_id={}, teammate_name={}, team_name={}",
        payload.base.session_id, payload.teammate_name, payload.team_name
    );

    let agent_name = std::env::var(AGENT_ENV_VAR).ok();
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    std::env::set_var("CONCLAUDE_TEAMMATE_NAME", &payload.teammate_name);
    std::env::set_var("CONCLAUDE_TEAM_NAME", &payload.team_name);

    let (config, config_path) = get_config().await?;
    let config_dir = get_config_dir(config_path);

    if !config.teammate_idle.commands.is_empty() {
        let matching_patterns =
            match_generic_patterns(&payload.teammate_name, &config.teammate_idle.commands)?;

        if !matching_patterns.is_empty() {
            println!(
                "Teammate '{}' matched patterns: {:?}",
                payload.teammate_name, matching_patterns
            );

            let commands =
                collect_teammate_idle_commands(&config.teammate_idle, &matching_patterns)?;

            if !commands.is_empty() {
                let env_vars = build_teammate_idle_env_vars(&payload, config_dir);
                let result =
                    execute_generic_commands(&commands, &env_vars, config_dir, "TeammateIdle")
                        .await?;
                if let Some(blocked_result) = result {
                    return Ok(blocked_result);
                }
            }
        } else {
            println!(
                "Teammate '{}' did not match any configured patterns",
                payload.teammate_name
            );
        }
    }

    send_notification(
        "TeammateIdle",
        "success",
        Some(&format!("Teammate '{}' idle", payload.teammate_name)),
    );

    Ok(HookResult::success())
}

/// Handles `TaskCompleted` hook events when a task is completed.
/// Can block completion if configured commands exit with code 2.
///
/// # Errors
///
/// Returns an error if payload reading, validation, or command execution fails.
pub async fn handle_task_completed() -> Result<HookResult> {
    let payload: TaskCompletedPayload = read_payload_from_stdin()?;

    validate_task_completed_payload(&payload).map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "Processing TaskCompleted hook: session_id={}, task_id={}, task_subject={}",
        payload.base.session_id, payload.task_id, payload.task_subject
    );

    let agent_name = std::env::var(AGENT_ENV_VAR).ok();
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    std::env::set_var("CONCLAUDE_TASK_ID", &payload.task_id);
    std::env::set_var("CONCLAUDE_TASK_SUBJECT", &payload.task_subject);
    std::env::set_var(
        "CONCLAUDE_TASK_DESCRIPTION",
        payload.task_description.as_deref().unwrap_or(""),
    );

    let (config, config_path) = get_config().await?;
    let config_dir = get_config_dir(config_path);

    if !config.task_completed.commands.is_empty() {
        let matching_patterns =
            match_generic_patterns(&payload.task_subject, &config.task_completed.commands)?;

        if !matching_patterns.is_empty() {
            println!(
                "Task '{}' matched patterns: {:?}",
                payload.task_subject, matching_patterns
            );

            let commands =
                collect_task_completed_commands(&config.task_completed, &matching_patterns)?;

            if !commands.is_empty() {
                let env_vars = build_task_completed_env_vars(&payload, config_dir);
                let result =
                    execute_generic_commands(&commands, &env_vars, config_dir, "TaskCompleted")
                        .await?;
                if let Some(blocked_result) = result {
                    return Ok(blocked_result);
                }
            }
        } else {
            println!(
                "Task '{}' did not match any configured patterns",
                payload.task_subject
            );
        }
    }

    send_notification(
        "TaskCompleted",
        "success",
        Some(&format!("Task '{}' completed", payload.task_subject)),
    );

    Ok(HookResult::success())
}

/// Handles `ConfigChange` hook events when configuration changes.
/// Can block changes (exit code 2) except for policy_settings source.
///
/// # Errors
///
/// Returns an error if payload reading, validation, or command execution fails.
pub async fn handle_config_change() -> Result<HookResult> {
    let payload: ConfigChangePayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

    let source_str = payload.source.to_string();

    println!(
        "Processing ConfigChange hook: session_id={}, source={}, file_path={:?}",
        payload.base.session_id, source_str, payload.file_path
    );

    let agent_name = std::env::var(AGENT_ENV_VAR).ok();
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    std::env::set_var("CONCLAUDE_CONFIG_SOURCE", &source_str);
    std::env::set_var(
        "CONCLAUDE_CONFIG_FILE_PATH",
        payload.file_path.as_deref().unwrap_or(""),
    );

    let (config, config_path) = get_config().await?;
    let config_dir = get_config_dir(config_path);

    if !config.config_change.commands.is_empty() {
        let matching_patterns =
            match_generic_patterns(&source_str, &config.config_change.commands)?;

        if !matching_patterns.is_empty() {
            println!(
                "Config change source '{}' matched patterns: {:?}",
                source_str, matching_patterns
            );

            let commands =
                collect_config_change_commands(&config.config_change, &matching_patterns)?;

            if !commands.is_empty() {
                let env_vars = build_config_change_env_vars(&payload, config_dir);

                // Policy settings cannot be blocked
                if payload.source == ConfigChangeSource::PolicySettings {
                    // Execute commands but ignore exit code 2 (don't block)
                    let _ =
                        execute_generic_commands(&commands, &env_vars, config_dir, "ConfigChange")
                            .await;
                } else {
                    let result =
                        execute_generic_commands(&commands, &env_vars, config_dir, "ConfigChange")
                            .await?;
                    if let Some(blocked_result) = result {
                        return Ok(blocked_result);
                    }
                }
            }
        }
    }

    send_notification(
        "ConfigChange",
        "success",
        Some(&format!("Config changed: source={}", source_str)),
    );

    Ok(HookResult::success())
}

/// Handles `WorktreeCreate` hook events when a git worktree needs to be created.
/// Returns the worktree path as a string, NOT as a HookResult JSON.
/// Falls back to `git worktree add` if no custom command is configured.
///
/// # Errors
///
/// Returns an error if payload reading, validation, or worktree creation fails.
pub async fn handle_worktree_create() -> Result<String> {
    let payload: WorktreeCreatePayload = read_payload_from_stdin()?;

    validate_worktree_create_payload(&payload).map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "Processing WorktreeCreate hook: session_id={}, name={}",
        payload.base.session_id, payload.name
    );

    let agent_name = std::env::var(AGENT_ENV_VAR).ok();
    if let Some(ref name) = agent_name {
        std::env::set_var("CONCLAUDE_AGENT_NAME", name);
    }

    std::env::set_var("CONCLAUDE_WORKTREE_NAME", &payload.name);

    // Try to load config for custom worktree command
    let config_result = get_config().await;

    if let Ok((config, config_path)) = config_result {
        let config_dir = get_config_dir(config_path);

        if let Some(ref command) = config.worktree_create.command {
            // Use custom command
            let timeout_secs = config.worktree_create.timeout.unwrap_or(60);

            let mut env_vars = HashMap::new();
            env_vars.insert(
                "CONCLAUDE_WORKTREE_NAME".to_string(),
                payload.name.clone(),
            );
            env_vars.insert(
                "CONCLAUDE_SESSION_ID".to_string(),
                payload.base.session_id.clone(),
            );
            env_vars.insert("CONCLAUDE_CWD".to_string(), payload.base.cwd.clone());
            env_vars.insert(
                "CONCLAUDE_HOOK_EVENT".to_string(),
                "WorktreeCreate".to_string(),
            );

            let child = TokioCommand::new("bash")
                .arg("-c")
                .arg(command)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .envs(&env_vars)
                .current_dir(config_dir)
                .spawn()
                .context("Failed to spawn worktree create command")?;

            let output = timeout(Duration::from_secs(timeout_secs), child.wait_with_output())
                .await
                .map_err(|_| {
                    anyhow::anyhow!(
                        "Worktree create command timed out after {} seconds",
                        timeout_secs
                    )
                })?
                .context("Failed to wait for worktree create command")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!(
                    "Worktree create command failed (exit code {}): {}",
                    output.status.code().unwrap_or(1),
                    stderr.trim()
                ));
            }

            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                return Err(anyhow::anyhow!(
                    "Worktree create command produced no output (expected worktree path on stdout)"
                ));
            }

            send_notification(
                "WorktreeCreate",
                "success",
                Some(&format!("Worktree created: {}", path)),
            );

            return Ok(path);
        }
    }

    // Fallback: use git worktree add
    let worktree_path = format!("../{}", payload.name);

    let child = TokioCommand::new("git")
        .args(["worktree", "add", &worktree_path])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&payload.base.cwd)
        .spawn()
        .context("Failed to spawn git worktree add")?;

    let output = timeout(Duration::from_secs(60), child.wait_with_output())
        .await
        .map_err(|_| anyhow::anyhow!("git worktree add timed out"))?
        .context("Failed to wait for git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "git worktree add failed: {}",
            stderr.trim()
        ));
    }

    // Resolve the path
    let resolved = PathBuf::from(&payload.base.cwd).join(&worktree_path);
    let result_path = resolved
        .canonicalize()
        .unwrap_or(resolved)
        .to_string_lossy()
        .to_string();

    send_notification(
        "WorktreeCreate",
        "success",
        Some(&format!("Worktree created: {}", result_path)),
    );

    Ok(result_path)
}

/// Special wrapper for WorktreeCreate that prints the path to stdout instead of JSON.
///
/// # Errors
///
/// Returns an error if worktree creation fails.
pub async fn handle_worktree_create_result() -> Result<()> {
    match handle_worktree_create().await {
        Ok(path) => {
            // Print the raw path to stdout (not JSON)
            print!("{}", path);
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod prompt_context_tests {
    use super::*;
    use crate::config::ContextInjectionRule;
    use std::fs;
    use tempfile::TempDir;

    // Tests for Task 4.2: Regex pattern matching

    #[test]
    fn test_regex_pattern_matching() {
        // Test 1: Simple string pattern matching
        let simple_rule = ContextInjectionRule {
            pattern: "sidebar".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        let simple_regex = compile_rule_pattern(&simple_rule).unwrap();
        assert!(
            simple_regex.is_match("update the sidebar"),
            "Should match 'sidebar' in phrase"
        );
        assert!(
            simple_regex.is_match("sidebar component"),
            "Should match 'sidebar' at start"
        );
        assert!(
            !simple_regex.is_match("side bar"),
            "Should not match 'side bar' (two words)"
        );
        assert!(
            !simple_regex.is_match("update the navigation"),
            "Should not match unrelated text"
        );

        // Test 2: Alternation pattern matching
        let alt_rule = ContextInjectionRule {
            pattern: "auth|login|authentication".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        let alt_regex = compile_rule_pattern(&alt_rule).unwrap();
        assert!(
            alt_regex.is_match("fix auth bug"),
            "Should match 'auth' alternative"
        );
        assert!(
            alt_regex.is_match("update login page"),
            "Should match 'login' alternative"
        );
        assert!(
            alt_regex.is_match("add authentication"),
            "Should match 'authentication' alternative"
        );
        assert!(
            !alt_regex.is_match("update the navbar"),
            "Should not match unrelated text"
        );

        // Test 3: Multiple patterns - both match
        let sidebar_regex = compile_rule_pattern(&simple_rule).unwrap();
        let auth_rule = ContextInjectionRule {
            pattern: "auth".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        let auth_regex = compile_rule_pattern(&auth_rule).unwrap();
        assert!(
            sidebar_regex.is_match("update the auth sidebar"),
            "Sidebar pattern should match"
        );
        assert!(
            auth_regex.is_match("update the auth sidebar"),
            "Auth pattern should match"
        );

        // Test 4: Multiple patterns - only one matches
        assert!(
            sidebar_regex.is_match("update the sidebar"),
            "Sidebar should match"
        );
        assert!(
            !auth_regex.is_match("update the sidebar"),
            "Auth should not match"
        );

        // Test 5: Multiple patterns - none match
        assert!(
            !sidebar_regex.is_match("update the navigation"),
            "Sidebar should not match"
        );
        assert!(
            !auth_regex.is_match("update the navigation"),
            "Auth should not match"
        );

        // Test 6: Invalid regex patterns return None
        let invalid_bracket = ContextInjectionRule {
            pattern: "[invalid".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        assert!(
            compile_rule_pattern(&invalid_bracket).is_none(),
            "Invalid bracket should return None"
        );

        let invalid_paren = ContextInjectionRule {
            pattern: "(unclosed".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };
        assert!(
            compile_rule_pattern(&invalid_paren).is_none(),
            "Unclosed paren should return None"
        );
    }

    #[test]
    fn test_regex_case_insensitive_with_flag_in_pattern() {
        let rule = ContextInjectionRule {
            pattern: "(?i)database".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: None,
        };

        let regex = compile_rule_pattern(&rule).unwrap();
        assert!(regex.is_match("DATABASE connection"));
        assert!(regex.is_match("database query"));
        assert!(regex.is_match("Database setup"));
    }

    #[test]
    fn test_regex_case_insensitive_with_config_field() {
        let rule = ContextInjectionRule {
            pattern: "database".to_string(),
            prompt: "Test".to_string(),
            enabled: Some(true),
            case_insensitive: Some(true),
        };

        let regex = compile_rule_pattern(&rule).unwrap();
        assert!(regex.is_match("DATABASE connection"));
        assert!(regex.is_match("database query"));
        assert!(regex.is_match("Database setup"));
    }

    #[test]
    fn test_expand_file_references_valid_file() {
        // Create temporary directory and file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "This is test content").unwrap();

        let prompt = format!("Read @{}", file_path.file_name().unwrap().to_str().unwrap());
        let expanded = expand_file_references(&prompt, temp_dir.path());

        assert_eq!(expanded, "Read This is test content");
    }

    #[test]
    fn test_expand_file_references_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let prompt = "Read @missing-file.md";
        let expanded = expand_file_references(prompt, temp_dir.path());

        // Missing file reference should be left as-is
        assert_eq!(expanded, "Read @missing-file.md");
    }

    #[test]
    fn test_expand_file_references_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file1, "Content 1").unwrap();
        fs::write(&file2, "Content 2").unwrap();

        let prompt = "Read @file1.md and @file2.md";
        let expanded = expand_file_references(prompt, temp_dir.path());

        assert_eq!(expanded, "Read Content 1 and Content 2");
    }

    #[test]
    fn test_expand_file_references_no_references() {
        let temp_dir = TempDir::new().unwrap();
        let prompt = "This is a normal prompt without file references";
        let expanded = expand_file_references(prompt, temp_dir.path());

        // Prompt should be unchanged
        assert_eq!(expanded, "This is a normal prompt without file references");
    }
}

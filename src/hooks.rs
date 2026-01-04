use crate::config::{
    extract_bash_commands, load_conclaude_config, ConclaudeConfig, SubagentStopConfig,
    UserPromptSubmitCommand,
};
use crate::gitignore::{find_git_root, is_path_git_ignored};
use crate::types::{
    validate_base_payload, validate_permission_request_payload, validate_subagent_start_payload,
    validate_subagent_stop_payload, HookResult, NotificationPayload, PermissionRequestPayload,
    PostToolUsePayload, PreCompactPayload, PreToolUsePayload, SessionEndPayload,
    SessionStartPayload, StopPayload, SubagentStartPayload, SubagentStopPayload,
    UserPromptSubmitPayload,
};
use anyhow::{Context, Result};
use glob::Pattern;
use notify_rust::{Notification, Urgency};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration};

/// Get the path to the agent session file for a given session.
pub fn get_agent_session_file_path(session_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("conclaude-agent-{}.json", session_id))
}

/// Write agent info to session file during SubagentStart.
///
/// # Errors
///
/// Returns an error if the session file cannot be written.
pub fn write_agent_session_file(session_id: &str, subagent_type: &str) -> std::io::Result<()> {
    let path = get_agent_session_file_path(session_id);
    let content = serde_json::json!({
        "subagent_type": subagent_type
    });
    fs::write(&path, content.to_string())
}

/// Read agent info from session file during PreToolUse.
/// Returns "main" if no session file exists (we're in the orchestrator session).
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

    // Detect current agent context from session file
    let current_agent = read_agent_from_session_file(&payload.base.session_id);

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

    // User prompt environment variable
    env_vars.insert("CONCLAUDE_USER_PROMPT".to_string(), payload.prompt.clone());

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

    if payload.prompt.is_empty() {
        return Err(anyhow::anyhow!("Missing required field: prompt"));
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
        if regex.is_match(&payload.prompt) {
            // Expand any @file references in the prompt
            let expanded_prompt = expand_file_references(&rule.prompt, config_dir);
            matching_contexts.push(expanded_prompt);
            matched_patterns.push(rule.pattern.clone());
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
        // Collect commands that match the prompt
        let commands = collect_user_prompt_submit_commands(
            &config.user_prompt_submit.commands,
            &payload.prompt,
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

    // Write agent session file for cross-process agent detection
    if let Err(e) = write_agent_session_file(&payload.base.session_id, &payload.subagent_type) {
        eprintln!("Warning: Failed to write agent session file: {}", e);
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

/// Extract the agent name (subagent_type) from the main transcript file
/// by finding the Task tool call that spawned this agent.
///
/// Returns None if the agent name cannot be found.
///
/// # Arguments
///
/// * `transcript_path` - Path to the main transcript file (JSONL format)
/// * `agent_id` - The agent ID to search for
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn extract_agent_name_from_transcript(
    transcript_path: &str,
    agent_id: &str,
) -> Result<Option<String>> {
    // Open the transcript file
    let file = match fs::File::open(transcript_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "Failed to open transcript file '{}' for agent name extraction: {}",
                transcript_path, e
            );
            return Ok(None);
        }
    };

    let reader = BufReader::new(file);

    // First pass: find the tool_result with matching agentId and get the tool_use_id
    let mut tool_use_id: Option<String> = None;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line from transcript: {}", e);
                continue;
            }
        };

        // Parse the JSON line
        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue, // Skip malformed lines
        };

        // Check if this is a tool_result with matching agentId
        if let Some(tool_use_result) = parsed.get("toolUseResult") {
            if let Some(result_agent_id) = tool_use_result.get("agentId").and_then(|v| v.as_str()) {
                if result_agent_id == agent_id {
                    // Found the matching tool result, extract tool_use_id
                    if let Some(message) = parsed.get("message") {
                        if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                            for item in content {
                                if let Some(use_id) =
                                    item.get("tool_use_id").and_then(|v| v.as_str())
                                {
                                    tool_use_id = Some(use_id.to_string());
                                    break;
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    let tool_use_id = match tool_use_id {
        Some(id) => id,
        None => {
            eprintln!(
                "Could not find tool_result with agentId '{}' in transcript",
                agent_id
            );
            return Ok(None);
        }
    };

    // Second pass: find the Task tool_use with matching id and extract subagent_type
    let file = fs::File::open(transcript_path)?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line from transcript: {}", e);
                continue;
            }
        };

        // Parse the JSON line
        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Check if this has a message.content array
        if let Some(message) = parsed.get("message") {
            if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                for item in content {
                    // Check if this is a tool_use with type "tool_use"
                    if item.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                        // Check if the id matches
                        if item.get("id").and_then(|v| v.as_str()) == Some(&tool_use_id) {
                            // Check if the name is "Task"
                            if item.get("name").and_then(|v| v.as_str()) == Some("Task") {
                                // Extract subagent_type from input
                                if let Some(input) = item.get("input") {
                                    if let Some(subagent_type) =
                                        input.get("subagent_type").and_then(|v| v.as_str())
                                    {
                                        return Ok(Some(subagent_type.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!(
        "Could not find Task tool_use with id '{}' in transcript",
        tool_use_id
    );
    Ok(None)
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

    // Extract agent name from main transcript
    let agent_name =
        extract_agent_name_from_transcript(&payload.base.transcript_path, &payload.agent_id)?;

    // Set environment variables for the subagent's information
    std::env::set_var("CONCLAUDE_AGENT_ID", &payload.agent_id);
    std::env::set_var(
        "CONCLAUDE_AGENT_TRANSCRIPT_PATH",
        &payload.agent_transcript_path,
    );

    // Set CONCLAUDE_AGENT_NAME to the extracted name, or fall back to agent_id
    let agent_name_value = agent_name.as_deref().unwrap_or(&payload.agent_id);
    std::env::set_var("CONCLAUDE_AGENT_NAME", agent_name_value);

    if let Some(name) = &agent_name {
        println!(
            "Extracted agent name '{}' for agent_id '{}'",
            name, payload.agent_id
        );
    } else {
        println!(
            "Could not extract agent name for agent_id '{}', using agent_id as fallback",
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

/// Handles `PreCompact` hook events before transcript compaction occurs.
///
/// # Errors
///
/// Returns an error if payload validation fails or configuration loading fails.
#[allow(clippy::unused_async)]
pub async fn handle_pre_compact() -> Result<HookResult> {
    let payload: PreCompactPayload = read_payload_from_stdin()?;

    validate_base_payload(&payload.base).map_err(|e| anyhow::anyhow!(e))?;

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

    // Detect current agent context from session file
    let current_agent = read_agent_from_session_file(&payload.base.session_id);

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

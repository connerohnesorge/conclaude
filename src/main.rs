// Testing GitHub Actions workflow fixes
mod config;
mod gitignore;
mod hooks;
mod schema;
mod types;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hooks::{
    handle_hook_result, handle_notification, handle_permission_request, handle_post_tool_use,
    handle_pre_compact, handle_pre_tool_use, handle_session_end, handle_session_start, handle_stop,
    handle_subagent_start, handle_subagent_stop, handle_user_prompt_submit,
};
use std::fs;
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Environment variable name for passing agent context to hook handlers
pub const AGENT_ENV_VAR: &str = "CONCLAUDE_AGENT";

/// Sets the agent name in an environment variable for hook handlers to access
fn set_agent_env(agent: Option<&str>) {
    if let Some(name) = agent {
        std::env::set_var(AGENT_ENV_VAR, name);
    }
}

/// Claude Code hook handler CLI tool that processes hook events and manages lifecycle hooks
#[derive(Parser)]
#[command(
    name = "conclaude",
    version = VERSION,
    about = "Claude Code Hook Handler - Processes hook events via JSON payloads from stdin",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize conclaude configuration and Claude Code hooks
    Init {
        /// Path for .conclaude.yaml file
        #[arg(long)]
        config_path: Option<String>,

        /// Path for .claude directory
        #[arg(long)]
        claude_path: Option<String>,

        /// Overwrite existing configuration files
        #[arg(short = 'f', long)]
        force: bool,

        /// Custom schema URL for YAML language server header
        #[arg(long)]
        schema_url: Option<String>,
    },
    /// Hook commands for Claude Code integration
    #[clap(name = "Hooks")]
    Hooks {
        #[command(subcommand)]
        command: HooksCommands,
    },
    /// Visualize file/directory settings from configuration
    Visualize {
        /// The specific rule to visualize (e.g., "uneditableFiles", "preventRootAdditions")
        #[arg(short, long)]
        rule: Option<String>,

        /// Show files that match the rule
        #[arg(long)]
        show_matches: bool,
    },
    /// Validate conclaude configuration file
    Validate {
        /// Path to configuration file to validate
        #[arg(long)]
        config_path: Option<String>,
    },
}

#[derive(Subcommand)]
enum HooksCommands {
    /// Process `PreToolUse` hook - fired before tool execution
    #[clap(name = "PreToolUse")]
    PreToolUse {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `PostToolUse` hook - fired after tool execution
    #[clap(name = "PostToolUse")]
    PostToolUse {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `PermissionRequest` hook - fired when tool requests permission
    #[clap(name = "PermissionRequest")]
    PermissionRequest {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process Notification hook - fired for system notifications
    #[clap(name = "Notification")]
    Notification {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `UserPromptSubmit` hook - fired when user submits input
    #[clap(name = "UserPromptSubmit")]
    UserPromptSubmit {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `SessionStart` hook - fired when session begins
    #[clap(name = "SessionStart")]
    SessionStart {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `SessionEnd` hook - fired when session terminates
    #[clap(name = "SessionEnd")]
    SessionEnd {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process Stop hook - fired when session terminates
    #[clap(name = "Stop")]
    Stop {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `SubagentStart` hook - fired when subagent begins
    #[clap(name = "SubagentStart")]
    SubagentStart {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `SubagentStop` hook - fired when subagent completes
    #[clap(name = "SubagentStop")]
    SubagentStop {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
    /// Process `PreCompact` hook - fired before transcript compaction
    #[clap(name = "PreCompact")]
    PreCompact {
        /// Optional agent name for context-aware hook execution
        #[clap(long)]
        agent: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            config_path,
            claude_path,
            force,
            schema_url,
        } => handle_init(config_path, claude_path, force, schema_url).await,
        Commands::Hooks { command } => match command {
            HooksCommands::PreToolUse { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_pre_tool_use).await
            }
            HooksCommands::PostToolUse { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_post_tool_use).await
            }
            HooksCommands::PermissionRequest { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_permission_request).await
            }
            HooksCommands::Notification { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_notification).await
            }
            HooksCommands::UserPromptSubmit { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_user_prompt_submit).await
            }
            HooksCommands::SessionStart { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_session_start).await
            }
            HooksCommands::SessionEnd { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_session_end).await
            }
            HooksCommands::Stop { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_stop).await
            }
            HooksCommands::SubagentStart { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_subagent_start).await
            }
            HooksCommands::SubagentStop { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_subagent_stop).await
            }
            HooksCommands::PreCompact { agent } => {
                set_agent_env(agent.as_deref());
                handle_hook_result(handle_pre_compact).await
            }
        },
        Commands::Visualize { rule, show_matches } => handle_visualize(rule, show_matches).await,
        Commands::Validate { config_path } => handle_validate(config_path).await,
    }
}

/// TypeScript interfaces for Claude Code settings structure
#[derive(serde::Serialize, serde::Deserialize)]
struct ClaudeHookConfig {
    #[serde(rename = "type")]
    config_type: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClaudeHookMatcher {
    #[serde(default)]
    matcher: String,
    hooks: Vec<ClaudeHookConfig>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClaudePermissions {
    allow: Vec<String>,
    deny: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClaudeSettings {
    #[serde(
        rename = "includeCoAuthoredBy",
        skip_serializing_if = "Option::is_none"
    )]
    include_co_authored_by: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<ClaudePermissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hooks: Option<std::collections::HashMap<String, Vec<ClaudeHookMatcher>>>,
}

/// Handles Init command to set up conclaude configuration and Claude Code hooks.
///
/// # Errors
///
/// Returns an error if directory access fails, file operations fail, or JSON serialization fails.
#[allow(clippy::unused_async)]
async fn handle_init(
    config_path: Option<String>,
    claude_path: Option<String>,
    force: bool,
    schema_url: Option<String>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let config_path = config_path.map_or_else(|| cwd.join(".conclaude.yaml"), PathBuf::from);
    let claude_path = claude_path.map_or_else(|| cwd.join(".claude"), PathBuf::from);
    let settings_path = claude_path.join("settings.json");

    println!("Initializing conclaude configuration...");

    // Check if config file exists
    if config_path.exists() && !force {
        eprintln!(
            "WARNING: Configuration file already exists: {}",
            config_path.display()
        );
        eprintln!("Use --force to overwrite existing configuration.");
        std::process::exit(1);
    }

    // Create .conclaude.yaml with YAML language server header
    let yaml_header = schema::generate_yaml_language_server_header(schema_url.as_deref());
    let config_content = format!("{}{}", yaml_header, config::generate_default_config());
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    println!(
        "[OK] Created configuration file with YAML language server support: {}",
        config_path.display()
    );
    let default_schema_url = schema::get_schema_url();
    let used_schema_url = schema_url.as_deref().unwrap_or(&default_schema_url);
    println!("   Schema URL: {used_schema_url}");

    // Create .claude directory if it doesn't exist
    fs::create_dir_all(&claude_path).with_context(|| {
        format!(
            "Failed to create .claude directory: {}",
            claude_path.display()
        )
    })?;

    // Handle settings.json
    let mut settings = if settings_path.exists() {
        let settings_content = fs::read_to_string(&settings_path).with_context(|| {
            format!("Failed to read settings file: {}", settings_path.display())
        })?;
        let settings: ClaudeSettings =
            serde_json::from_str(&settings_content).with_context(|| {
                format!("Failed to parse settings file: {}", settings_path.display())
            })?;
        println!("Found existing Claude settings, updating hooks...");
        settings
    } else {
        println!("Creating Claude Code settings...");
        ClaudeSettings {
            include_co_authored_by: None,
            permissions: Some(ClaudePermissions {
                allow: Vec::new(),
                deny: Vec::new(),
            }),
            hooks: Some(std::collections::HashMap::new()),
        }
    };

    // Define all hook types and their commands
    // Note: SubagentStart and SubagentStop are NOT included here because
    // agent-specific hooks now live in agent frontmatter files (.claude/agents/*.md)
    let hook_types = [
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "PermissionRequest",
        "Notification",
        "Stop",
        "PreCompact",
        "SessionStart",
        "SessionEnd",
    ];

    // Add hook configurations
    let hooks = settings
        .hooks
        .get_or_insert_with(std::collections::HashMap::new);
    for hook_type in &hook_types {
        hooks.insert(
            (*hook_type).to_string(),
            vec![ClaudeHookMatcher {
                matcher: String::new(),
                hooks: vec![ClaudeHookConfig {
                    config_type: "command".to_string(),
                    command: format!("conclaude Hooks {hook_type}"),
                    prompt: None,
                    timeout: Some(600),
                }],
            }],
        );
    }

    // Write updated settings
    let settings_json =
        serde_json::to_string_pretty(&settings).context("Failed to serialize settings to JSON")?;
    fs::write(&settings_path, settings_json)
        .with_context(|| format!("Failed to write settings file: {}", settings_path.display()))?;

    println!(
        "[OK] Updated Claude Code settings: {}",
        settings_path.display()
    );

    println!("Conclaude initialization complete!");
    println!("Configured hooks:");
    for hook_type in &hook_types {
        println!("   - {hook_type}");
    }
    println!("You can now use Claude Code with conclaude hook handling.");

    // Inject hooks into agent files
    println!("\nSearching for agent files...");
    let agents_path = claude_path.join("agents");
    if agents_path.exists() {
        inject_agent_hooks(&agents_path)?;
    } else {
        println!("No agents directory found, skipping agent hook injection.");
    }

    Ok(())
}

/// Discover agent markdown files in the .claude/agents directory
///
/// # Errors
///
/// Returns an error if glob pattern is invalid or directory access fails.
fn discover_agent_files(agents_dir: &Path) -> Result<Vec<PathBuf>> {
    let pattern = agents_dir.join("**/*.md");
    let pattern_str = pattern
        .to_str()
        .context("Failed to convert path to string")?;

    let mut agent_files = Vec::new();
    for entry in glob::glob(pattern_str).context("Failed to create glob pattern")? {
        match entry {
            Ok(path) => agent_files.push(path),
            Err(e) => eprintln!("Warning: Failed to read path: {e}"),
        }
    }

    Ok(agent_files)
}

/// Parse agent frontmatter from markdown content
///
/// Returns (frontmatter_yaml, markdown_body)
///
/// # Errors
///
/// Returns an error if YAML parsing fails.
fn parse_agent_frontmatter(content: &str) -> Result<Option<(serde_yaml::Value, String)>> {
    // Check if content starts with ---
    if !content.starts_with("---") {
        return Ok(None);
    }

    // Find the closing --- (must be at start of line)
    let after_first = &content[3..];

    // Look for \n---\n or \n--- at end of string
    let end_pos = if let Some(pos) = after_first.find("\n---\n") {
        Some(pos)
    } else if after_first.ends_with("\n---") {
        Some(after_first.len() - 4)
    } else {
        None
    };

    if let Some(end_pos) = end_pos {
        let yaml_str = after_first[..end_pos].trim();
        let body = if after_first[end_pos..].starts_with("\n---\n") {
            &after_first[end_pos + 5..] // Skip past \n---\n
        } else {
            "" // No body after frontmatter
        };

        // Try to parse the YAML - if it fails, return None to skip the file
        match serde_yaml::from_str(yaml_str) {
            Ok(yaml_value) => Ok(Some((yaml_value, body.to_string()))),
            Err(e) => {
                // Return error with context
                Err(anyhow::anyhow!("Failed to parse agent frontmatter YAML: {}", e))
            }
        }
    } else {
        Ok(None)
    }
}

/// Generate hooks configuration for an agent
fn generate_agent_hooks(agent_name: &str) -> serde_yaml::Value {
    use serde_yaml::{Mapping, Value};

    let hook_types = [
        ("PreToolUse", true),      // needs matcher
        ("PostToolUse", true),     // needs matcher
        ("Stop", false),
        ("SessionStart", false),
        ("SessionEnd", false),
        ("Notification", true),    // needs matcher
        ("PreCompact", false),
        ("PermissionRequest", true), // needs matcher
        ("UserPromptSubmit", true),  // needs matcher
    ];

    let mut hooks_map = Mapping::new();

    for (hook_type, needs_matcher) in &hook_types {
        let mut hook_entry = Mapping::new();

        if *needs_matcher {
            hook_entry.insert(
                Value::String("matcher".to_string()),
                Value::String(String::new()),
            );
        }

        let mut hook_config = Mapping::new();
        hook_config.insert(
            Value::String("type".to_string()),
            Value::String("command".to_string()),
        );
        hook_config.insert(
            Value::String("command".to_string()),
            Value::String(format!("conclaude Hooks {hook_type} --agent {agent_name}")),
        );

        let hooks_array = vec![Value::Mapping(hook_config)];
        hook_entry.insert(
            Value::String("hooks".to_string()),
            Value::Sequence(hooks_array),
        );

        hooks_map.insert(
            Value::String(hook_type.to_string()),
            Value::Sequence(vec![Value::Mapping(hook_entry)]),
        );
    }

    Value::Mapping(hooks_map)
}

/// Inject hooks into a single agent file
///
/// # Errors
///
/// Returns an error if file operations or YAML serialization fails.
fn inject_agent_hooks_into_file(agent_path: &Path) -> Result<()> {
    // Read the file
    let content = fs::read_to_string(agent_path)
        .with_context(|| format!("Failed to read agent file: {}", agent_path.display()))?;

    // Parse frontmatter
    let (mut frontmatter, body) = match parse_agent_frontmatter(&content)? {
        Some((fm, b)) => (fm, b),
        None => {
            eprintln!(
                "Warning: Agent file has no frontmatter, skipping: {}",
                agent_path.display()
            );
            return Ok(());
        }
    };

    // Get agent name from frontmatter or filename
    let agent_name = if let Some(name) = frontmatter.get("name").and_then(|v| v.as_str()) {
        name.to_string()
    } else {
        // Derive from filename
        let name = agent_path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Failed to extract agent name from filename")?
            .to_string();
        eprintln!(
            "Warning: Agent file has no 'name' field, using filename: {}",
            name
        );
        name
    };

    // Check if hooks already exist
    if frontmatter.get("hooks").is_some() {
        println!("   Agent '{}' already has hooks, skipping.", agent_name);
        return Ok(());
    }

    // Generate and inject hooks
    let hooks = generate_agent_hooks(&agent_name);
    if let serde_yaml::Value::Mapping(ref mut map) = frontmatter {
        map.insert(
            serde_yaml::Value::String("hooks".to_string()),
            hooks,
        );
    }

    // Serialize frontmatter back to YAML
    let yaml_str = serde_yaml::to_string(&frontmatter)
        .context("Failed to serialize agent frontmatter")?;

    // Reconstruct the file
    let new_content = format!("---\n{}---\n{}", yaml_str, body);

    // Write back to file
    fs::write(agent_path, new_content)
        .with_context(|| format!("Failed to write agent file: {}", agent_path.display()))?;

    println!("   [OK] Injected hooks into agent: {}", agent_name);

    Ok(())
}

/// Inject hooks into all agent files in the agents directory
///
/// # Errors
///
/// Returns an error if agent discovery or injection fails.
fn inject_agent_hooks(agents_dir: &Path) -> Result<()> {
    let agent_files = discover_agent_files(agents_dir)?;

    if agent_files.is_empty() {
        println!("No agent files found in {}", agents_dir.display());
        return Ok(());
    }

    println!("Found {} agent file(s):", agent_files.len());
    for file in &agent_files {
        println!("   - {}", file.display());
    }

    println!("\nInjecting hooks into agent files...");
    let mut success_count = 0;
    let mut error_count = 0;

    for agent_file in &agent_files {
        match inject_agent_hooks_into_file(agent_file) {
            Ok(()) => success_count += 1,
            Err(e) => {
                eprintln!("   [ERROR] Failed to inject hooks into {}: {}", agent_file.display(), e);
                error_count += 1;
            }
        }
    }

    println!("\nAgent hook injection complete!");
    println!("   Success: {}", success_count);
    if error_count > 0 {
        println!("   Errors: {} (see warnings above)", error_count);
    }

    Ok(())
}

/// Handles Visualize command to display file/directory settings from configuration.
///
/// # Errors
///
/// Returns an error if configuration loading fails, directory access fails, or glob pattern creation fails.
#[allow(clippy::too_many_lines)]
#[allow(clippy::unused_async)]
async fn handle_visualize(rule: Option<String>, show_matches: bool) -> Result<()> {
    use glob::Pattern;
    use walkdir::WalkDir;

    println!("Visualizing configuration rules...");

    let (config, _config_path) = config::load_conclaude_config(None)
        .await
        .context("Failed to load configuration")?;

    if let Some(rule_name) = rule {
        match rule_name.as_str() {
            "uneditableFiles" => {
                println!("Uneditable Files:");
                if config.pre_tool_use.uneditable_files.is_empty() {
                    println!("   No uneditable files configured");
                } else {
                    for rule in &config.pre_tool_use.uneditable_files {
                        let pattern_str = rule.pattern();
                        println!("   Pattern: {pattern_str}");
                        if let Some(msg) = rule.message() {
                            println!("   Message: {msg}");
                        }

                        if show_matches {
                            let pattern = Pattern::new(pattern_str)?;
                            println!("   Matching files:");
                            let mut found = false;

                            for entry in WalkDir::new(".")
                                .into_iter()
                                .filter_map(std::result::Result::ok)
                            {
                                if entry.file_type().is_file() {
                                    let path = entry.path();
                                    if pattern.matches(&path.to_string_lossy()) {
                                        println!("      - {}", path.display());
                                        found = true;
                                    }
                                }
                            }

                            if !found {
                                println!("      (no matching files found)");
                            }
                        }
                    }
                }
            }
            "preventRootAdditions" => {
                println!(
                    "Prevent Root Additions: {}",
                    config.pre_tool_use.prevent_root_additions
                );
                if config.pre_tool_use.prevent_root_additions && show_matches {
                    println!("   Root directory contents:");
                    for entry in (fs::read_dir(".")?).flatten() {
                        println!("      - {}", entry.file_name().to_string_lossy());
                    }
                }
            }
            "toolUsageValidation" => {
                println!("Tool Usage Validation Rules:");
                if config.pre_tool_use.tool_usage_validation.is_empty() {
                    println!("   No tool usage validation rules configured");
                } else {
                    for rule in &config.pre_tool_use.tool_usage_validation {
                        println!(
                            "   Tool: {} | Pattern: {} | Action: {}",
                            rule.tool, rule.pattern, rule.action
                        );
                        if let Some(msg) = &rule.message {
                            println!("      Message: {msg}");
                        }
                    }
                }
            }
            _ => {
                eprintln!("[ERROR] Unknown rule: {rule_name}");
                println!("Available rules:");
                println!("   - uneditableFiles");
                println!("   - preventRootAdditions");
                println!("   - toolUsageValidation");
            }
        }
    } else {
        // Show all rules overview
        println!("Configuration Overview:");
        println!(
            "Prevent Root Additions: {}",
            config.pre_tool_use.prevent_root_additions
        );
        println!(
            "Uneditable Files: {} patterns",
            config.pre_tool_use.uneditable_files.len()
        );
        println!(
            "Tool Usage Validation: {} rules",
            config.pre_tool_use.tool_usage_validation.len()
        );
        println!("Infinite Mode: {}", config.stop.infinite);

        println!("Use --rule <rule-name> to see details for a specific rule");
        println!("Use --show-matches to see which files match the patterns");
    }

    Ok(())
}

/// Handles Validate command to validate conclaude configuration file.
///
/// # Errors
///
/// Returns an error if configuration loading or validation fails.
#[allow(clippy::unused_async)]
async fn handle_validate(config_path: Option<String>) -> Result<()> {
    println!("Validating conclaude configuration...");

    // Load and validate configuration
    let result = if let Some(custom_path) = config_path {
        let path = PathBuf::from(&custom_path);

        // First, check if the path exists
        if !path.exists() {
            anyhow::bail!("Path not found: {}", path.display());
        }

        // Determine path type using filesystem queries
        if path.is_file() {
            // It's a regular file - load it directly
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;

            // Parse and validate using shared logic with enhanced error messages
            let config = config::parse_and_validate_config(&content, &path)?;

            Ok((config, path))
        } else if path.is_dir() {
            // It's a directory - use the standard search from that directory
            config::load_conclaude_config(Some(&path)).await
        } else {
            // Not a regular file or directory
            anyhow::bail!(
                "Path is not a regular file or directory: {}",
                path.display()
            );
        }
    } else {
        // No custom path, use standard search from current directory
        config::load_conclaude_config(None).await
    };

    match result {
        Ok((config, found_path)) => {
            println!("[OK] Configuration is valid!");
            println!("   Config file: {}", found_path.display());
            println!();
            println!("Configuration summary:");
            println!(
                "   Prevent root additions: {}",
                config.pre_tool_use.prevent_root_additions
            );
            println!(
                "   Uneditable files: {} pattern(s)",
                config.pre_tool_use.uneditable_files.len()
            );
            println!(
                "   Tool usage validation: {} rule(s)",
                config.pre_tool_use.tool_usage_validation.len()
            );
            println!("   Infinite mode: {}", config.stop.infinite);
            println!("   Notifications enabled: {}", config.notifications.enabled);
            Ok(())
        }
        Err(e) => {
            eprintln!("[ERROR] Configuration validation failed:\n");
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

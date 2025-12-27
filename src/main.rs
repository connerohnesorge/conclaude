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
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    PreToolUse,
    /// Process `PostToolUse` hook - fired after tool execution
    #[clap(name = "PostToolUse")]
    PostToolUse,
    /// Process `PermissionRequest` hook - fired when tool requests permission
    #[clap(name = "PermissionRequest")]
    PermissionRequest,
    /// Process Notification hook - fired for system notifications
    #[clap(name = "Notification")]
    Notification,
    /// Process `UserPromptSubmit` hook - fired when user submits input
    #[clap(name = "UserPromptSubmit")]
    UserPromptSubmit,
    /// Process `SessionStart` hook - fired when session begins
    #[clap(name = "SessionStart")]
    SessionStart,
    /// Process `SessionEnd` hook - fired when session terminates
    #[clap(name = "SessionEnd")]
    SessionEnd,
    /// Process Stop hook - fired when session terminates
    #[clap(name = "Stop")]
    Stop,
    /// Process `SubagentStart` hook - fired when subagent begins
    #[clap(name = "SubagentStart")]
    SubagentStart,
    /// Process `SubagentStop` hook - fired when subagent completes
    #[clap(name = "SubagentStop")]
    SubagentStop,
    /// Process `PreCompact` hook - fired before transcript compaction
    #[clap(name = "PreCompact")]
    PreCompact,
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
            HooksCommands::PreToolUse => handle_hook_result(handle_pre_tool_use).await,
            HooksCommands::PostToolUse => handle_hook_result(handle_post_tool_use).await,
            HooksCommands::PermissionRequest => handle_hook_result(handle_permission_request).await,
            HooksCommands::Notification => handle_hook_result(handle_notification).await,
            HooksCommands::UserPromptSubmit => handle_hook_result(handle_user_prompt_submit).await,
            HooksCommands::SessionStart => handle_hook_result(handle_session_start).await,
            HooksCommands::SessionEnd => handle_hook_result(handle_session_end).await,
            HooksCommands::Stop => handle_hook_result(handle_stop).await,
            HooksCommands::SubagentStart => handle_hook_result(handle_subagent_start).await,
            HooksCommands::SubagentStop => handle_hook_result(handle_subagent_stop).await,
            HooksCommands::PreCompact => handle_hook_result(handle_pre_compact).await,
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
    let hook_types = [
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "PermissionRequest",
        "Notification",
        "Stop",
        "SubagentStart",
        "SubagentStop",
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

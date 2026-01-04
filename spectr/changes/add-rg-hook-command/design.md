# Design: Ripgrep Hook Command Integration

## Context

This document describes the technical implementation of the `rg` hook command type, which integrates ripgrep's library crates directly into conclaude for declarative pattern-based file content validation.

### Stakeholders

- **Claude Code users**: Benefit from simpler, more readable hook configurations
- **Conclaude maintainers**: Need to understand library integration patterns
- **Security teams**: Pattern-based validation for secrets detection, code quality

### Constraints

- Must not require external `rg` binary installation
- Must integrate with existing hook execution pipeline
- Must support all existing output limiting and display options
- Binary size increase should be reasonable (~2-3MB)
- Search performance should be comparable to native ripgrep

## Goals / Non-Goals

### Goals

- Provide declarative pattern matching as alternative to shell commands
- Expose comprehensive ripgrep options through configuration
- Integrate seamlessly with existing hook infrastructure
- Enable match counting with constraint validation
- Support both stop and subagentStop hooks

### Non-Goals

- Replace all `run` command use cases (shell commands still useful)
- Implement ripgrep's full CLI interface (curated subset only)
- Support PCRE2 regex engine (Rust regex only)
- Provide interactive/streaming output

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Configuration Layer                          │
├─────────────────────────────────────────────────────────────────────┤
│  .conclaude.yaml                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │
│  │   StopCommand   │  │ SubagentStop    │  │     RgConfig        │  │
│  │   (run | rg)    │  │ Command         │  │  (pattern, files,   │  │
│  │                 │  │ (run | rg)      │  │   options...)       │  │
│  └────────┬────────┘  └────────┬────────┘  └──────────┬──────────┘  │
└───────────┼────────────────────┼─────────────────────┼──────────────┘
            │                    │                     │
            ▼                    ▼                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Execution Layer                              │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                     RgSearcher                               │    │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐ │    │
│  │  │ WalkBuilder │  │RegexMatcher  │  │   SearcherBuilder   │ │    │
│  │  │  (ignore)   │  │ (grep-regex) │  │   (grep-searcher)   │ │    │
│  │  └──────┬──────┘  └──────┬───────┘  └──────────┬──────────┘ │    │
│  │         │                │                     │            │    │
│  │         ▼                ▼                     ▼            │    │
│  │  ┌──────────────────────────────────────────────────────┐   │    │
│  │  │                   CountingSink                        │   │    │
│  │  │  - Collects matches and context lines                 │   │    │
│  │  │  - Counts lines or occurrences                        │   │    │
│  │  │  - Respects maxOutputLines limit                      │   │    │
│  │  └──────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Constraint Layer                             │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │              ConstraintEvaluator                             │    │
│  │  - Compares count against max/min/equal                      │    │
│  │  - Formats violation messages                                │    │
│  │  - Applies action (block vs warn)                            │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Hook Result                                  │
├─────────────────────────────────────────────────────────────────────┤
│  HookResult { blocked: bool, message: Option<String>, ... }         │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Structures

### Configuration Structs

```rust
/// Configuration for ripgrep-based pattern matching
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, FieldList)]
#[serde(deny_unknown_fields)]
pub struct RgConfig {
    // === Required Fields ===

    /// Regex pattern to search for
    pub pattern: String,

    /// Glob pattern for file matching (e.g., "**/*.rs")
    pub files: String,

    // === Constraint Fields (mutually exclusive) ===

    /// Maximum allowed matches (default when none specified: 0)
    #[serde(default)]
    pub max: Option<u64>,

    /// Minimum required matches
    #[serde(default)]
    pub min: Option<u64>,

    /// Exact match count required
    #[serde(default)]
    pub equal: Option<u64>,

    // === Regex Options ===

    /// Case insensitive matching
    #[serde(default, rename = "ignoreCase")]
    pub ignore_case: bool,

    /// Smart case: auto-detect from pattern
    #[serde(default, rename = "smartCase")]
    pub smart_case: bool,

    /// Word boundary matching (like \b but smarter)
    #[serde(default)]
    pub word: bool,

    /// Treat pattern as literal string, not regex
    #[serde(default, rename = "fixedStrings")]
    pub fixed_strings: bool,

    /// Multi-line mode: ^ and $ match line boundaries
    #[serde(default, rename = "multiLine")]
    pub multi_line: bool,

    /// Match entire line only
    #[serde(default, rename = "wholeLine")]
    pub whole_line: bool,

    /// Dot matches newline characters
    #[serde(default, rename = "dotMatchesNewLine")]
    pub dot_matches_new_line: bool,

    /// Enable Unicode character classes (default: true)
    #[serde(default = "default_true")]
    pub unicode: bool,

    // === File Walking Options ===

    /// Maximum directory depth
    #[serde(default, rename = "maxDepth")]
    pub max_depth: Option<usize>,

    /// Include hidden files (starting with .)
    #[serde(default)]
    pub hidden: bool,

    /// Follow symbolic links
    #[serde(default, rename = "followLinks")]
    pub follow_links: bool,

    /// Skip files larger than N bytes
    #[serde(default, rename = "maxFilesize")]
    pub max_filesize: Option<u64>,

    /// Respect .gitignore files (default: true)
    #[serde(default = "default_true", rename = "gitIgnore")]
    pub git_ignore: bool,

    /// Respect .ignore files (default: true)
    #[serde(default = "default_true")]
    pub ignore: bool,

    /// Read parent directory ignore files (default: true)
    #[serde(default = "default_true")]
    pub parents: bool,

    /// Don't cross filesystem boundaries
    #[serde(default, rename = "sameFileSystem")]
    pub same_file_system: bool,

    /// Number of parallel search threads
    #[serde(default)]
    pub threads: Option<usize>,

    /// File type filters (e.g., ["rust", "js"])
    #[serde(default)]
    pub types: Vec<String>,

    // === Search Options ===

    /// Lines of context before and after matches
    #[serde(default)]
    pub context: usize,

    /// How to count matches: "lines" or "occurrences"
    #[serde(default, rename = "countMode")]
    pub count_mode: CountMode,

    /// Show non-matching lines (invert match)
    #[serde(default, rename = "invertMatch")]
    pub invert_match: bool,
}

/// How matches are counted
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CountMode {
    /// Count lines containing matches (default)
    #[default]
    Lines,
    /// Count every match occurrence
    Occurrences,
}

/// Action to take when constraint is violated
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CommandAction {
    /// Block the hook (default)
    #[default]
    Block,
    /// Log warning but continue
    Warn,
}

/// Modified StopCommand with rg support
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, FieldList)]
#[serde(deny_unknown_fields)]
pub struct StopCommand {
    /// Shell command to execute (mutually exclusive with rg)
    #[serde(default)]
    pub run: Option<String>,

    /// Ripgrep configuration (mutually exclusive with run)
    #[serde(default)]
    pub rg: Option<RgConfig>,

    /// Action on failure: "block" (default) or "warn"
    #[serde(default)]
    pub action: CommandAction,

    // ... existing fields unchanged ...
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default = "default_option_true", rename = "showCommand")]
    pub show_command: Option<bool>,
    #[serde(default, rename = "showStdout")]
    pub show_stdout: Option<bool>,
    #[serde(default, rename = "showStderr")]
    pub show_stderr: Option<bool>,
    #[serde(default, rename = "maxOutputLines")]
    pub max_output_lines: Option<u32>,
    #[serde(default)]
    pub timeout: Option<u64>,
}
```

### Runtime Structs

```rust
/// Result of a ripgrep search
#[derive(Debug)]
pub struct RgSearchResult {
    /// Total match count (lines or occurrences based on mode)
    pub count: u64,
    /// Collected output lines (limited by maxOutputLines)
    pub output_lines: Vec<String>,
    /// Number of lines omitted due to limit
    pub lines_omitted: usize,
    /// Per-file match counts
    pub file_counts: HashMap<PathBuf, u64>,
    /// Any errors encountered during search
    pub errors: Vec<String>,
    /// Number of files searched
    pub files_searched: u64,
}

/// Constraint type for validation
#[derive(Debug, Clone, Copy)]
pub enum Constraint {
    Max(u64),
    Min(u64),
    Equal(u64),
}

impl Constraint {
    /// Evaluate constraint against count
    pub fn evaluate(&self, count: u64) -> ConstraintResult {
        match self {
            Constraint::Max(n) => {
                if count <= *n {
                    ConstraintResult::Pass
                } else {
                    ConstraintResult::Fail {
                        message: format!(
                            "Found {} matches, maximum allowed is {}",
                            count, n
                        ),
                    }
                }
            }
            Constraint::Min(n) => {
                if count >= *n {
                    ConstraintResult::Pass
                } else {
                    ConstraintResult::Fail {
                        message: format!(
                            "Found {} matches, minimum required is {}",
                            count, n
                        ),
                    }
                }
            }
            Constraint::Equal(n) => {
                if count == *n {
                    ConstraintResult::Pass
                } else {
                    ConstraintResult::Fail {
                        message: format!(
                            "Found {} matches, expected exactly {}",
                            count, n
                        ),
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ConstraintResult {
    Pass,
    Fail { message: String },
}
```

## Library Integration Details

### 1. Regex Matcher Configuration

The `grep-regex` crate provides `RegexMatcherBuilder` for constructing matchers:

```rust
use grep_regex::RegexMatcherBuilder;
use grep_matcher::Matcher;

fn build_matcher(config: &RgConfig) -> Result<RegexMatcher> {
    let mut builder = RegexMatcherBuilder::new();

    // Apply regex options
    builder
        .case_insensitive(config.ignore_case)
        .case_smart(config.smart_case)
        .word(config.word)
        .fixed_strings(config.fixed_strings)
        .multi_line(config.multi_line)
        .whole_line(config.whole_line)
        .dot_matches_new_line(config.dot_matches_new_line)
        .unicode(config.unicode);

    // Build the matcher
    builder.build(&config.pattern)
        .map_err(|e| anyhow!("Invalid regex pattern '{}': {}", config.pattern, e))
}
```

### 2. File Walker Configuration

The `ignore` crate provides `WalkBuilder` for file traversal:

```rust
use ignore::{WalkBuilder, types::TypesBuilder};
use std::path::Path;

fn build_walker(config: &RgConfig, base_path: &Path) -> Result<Walk> {
    let mut builder = WalkBuilder::new(base_path);

    // Apply walking options
    builder
        .hidden(!config.hidden)           // hidden() means "process hidden"
        .git_ignore(config.git_ignore)
        .git_global(config.git_ignore)    // Also respect global gitignore
        .git_exclude(config.git_ignore)   // Also respect .git/info/exclude
        .ignore(config.ignore)
        .parents(config.parents)
        .follow_links(config.follow_links)
        .same_file_system(config.same_file_system);

    // Optional limits
    if let Some(depth) = config.max_depth {
        builder.max_depth(Some(depth));
    }
    if let Some(size) = config.max_filesize {
        builder.max_filesize(Some(size));
    }
    if let Some(threads) = config.threads {
        builder.threads(threads);
    }

    // File type filtering
    if !config.types.is_empty() {
        let mut types_builder = TypesBuilder::new();
        types_builder.add_defaults(); // Load ripgrep's built-in types

        for type_name in &config.types {
            types_builder.select(type_name);
        }

        let types = types_builder.build()
            .map_err(|e| anyhow!("Invalid file type: {}", e))?;
        builder.types(types);
    }

    // Add glob override for files pattern
    let mut overrides = ignore::overrides::OverrideBuilder::new(base_path);
    overrides.add(&config.files)
        .map_err(|e| anyhow!("Invalid glob pattern '{}': {}", config.files, e))?;
    builder.overrides(overrides.build()?);

    Ok(builder.build())
}
```

### 3. Searcher Configuration

The `grep-searcher` crate provides `SearcherBuilder` for search execution:

```rust
use grep_searcher::{SearcherBuilder, BinaryDetection, Searcher};

fn build_searcher(config: &RgConfig) -> Searcher {
    let mut builder = SearcherBuilder::new();

    builder
        .line_number(true)  // Always track line numbers
        .binary_detection(BinaryDetection::quit(0x00))  // Skip binary files
        .before_context(config.context)
        .after_context(config.context);

    builder.build()
}
```

### 4. Custom Sink Implementation

The `Sink` trait from `grep-searcher` handles match collection:

```rust
use grep_searcher::{Searcher, Sink, SinkMatch, SinkContext, SinkFinish, SinkError};
use std::io;

/// Sink that counts matches and collects output
pub struct CountingSink {
    /// Current match count
    count: u64,
    /// Count mode
    count_mode: CountMode,
    /// Collected output lines
    output: Vec<String>,
    /// Maximum output lines to collect
    max_lines: Option<usize>,
    /// Lines omitted due to limit
    lines_omitted: usize,
    /// Current file path
    current_file: PathBuf,
    /// Matcher for occurrence counting
    matcher: RegexMatcher,
}

impl CountingSink {
    pub fn new(
        count_mode: CountMode,
        max_lines: Option<usize>,
        matcher: RegexMatcher,
    ) -> Self {
        Self {
            count: 0,
            count_mode,
            output: Vec::new(),
            max_lines,
            lines_omitted: 0,
            current_file: PathBuf::new(),
            matcher,
        }
    }

    pub fn set_file(&mut self, path: PathBuf) {
        self.current_file = path;
    }

    pub fn into_result(self) -> (u64, Vec<String>, usize) {
        (self.count, self.output, self.lines_omitted)
    }

    fn add_output_line(&mut self, line: String) {
        if let Some(max) = self.max_lines {
            if self.output.len() >= max {
                self.lines_omitted += 1;
                return;
            }
        }
        self.output.push(line);
    }
}

impl Sink for CountingSink {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        // Count based on mode
        match self.count_mode {
            CountMode::Lines => {
                self.count += 1;
            }
            CountMode::Occurrences => {
                // Count all matches in this line
                let line = mat.bytes();
                let mut count = 0u64;
                self.matcher.find_iter(line, |_| {
                    count += 1;
                    true // Continue finding
                }).ok();
                self.count += count;
            }
        }

        // Collect output
        let line_num = mat.line_number().unwrap_or(0);
        let content = String::from_utf8_lossy(mat.bytes());
        let output_line = format!(
            "{}:{}:{}",
            self.current_file.display(),
            line_num,
            content.trim_end()
        );
        self.add_output_line(output_line);

        Ok(true) // Continue searching
    }

    fn context(
        &mut self,
        _searcher: &Searcher,
        ctx: &SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
        // Collect context lines for output
        let line_num = ctx.line_number().unwrap_or(0);
        let content = String::from_utf8_lossy(ctx.bytes());
        let prefix = match ctx.kind() {
            SinkContextKind::Before => "-",
            SinkContextKind::After => "+",
            SinkContextKind::Other => " ",
        };
        let output_line = format!(
            "{}:{}:{}{}",
            self.current_file.display(),
            line_num,
            prefix,
            content.trim_end()
        );
        self.add_output_line(output_line);

        Ok(true)
    }

    fn finish(
        &mut self,
        _searcher: &Searcher,
        _finish: &SinkFinish,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
```

## Main Search Execution

```rust
/// Execute an rg command and return the result
pub async fn execute_rg_command(
    config: &RgConfig,
    config_dir: &Path,
    timeout: Option<Duration>,
) -> Result<RgSearchResult> {
    // Build components
    let matcher = build_matcher(config)?;
    let walker = build_walker(config, config_dir)?;
    let searcher = build_searcher(config);

    // Determine max output lines
    let max_lines = config.max_output_lines.map(|n| n as usize);

    // Create sink
    let mut sink = CountingSink::new(
        config.count_mode,
        max_lines,
        matcher.clone(),
    );

    // Track files
    let mut files_searched = 0u64;
    let mut file_counts: HashMap<PathBuf, u64> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut files_matched = 0usize;

    // Execute search with timeout
    let search_future = async {
        for entry in walker {
            match entry {
                Ok(entry) => {
                    // Skip directories
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        continue;
                    }

                    let path = entry.path().to_path_buf();
                    files_searched += 1;

                    // Set current file for output formatting
                    sink.set_file(path.clone());

                    // Search the file
                    let count_before = sink.count;
                    match searcher.search_path(&matcher, &path, &mut sink) {
                        Ok(()) => {
                            let matches_in_file = sink.count - count_before;
                            if matches_in_file > 0 {
                                file_counts.insert(path, matches_in_file);
                                files_matched += 1;
                            }
                        }
                        Err(e) => {
                            // Log error but continue
                            errors.push(format!("{}: {}", path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Walk error: {}", e));
                }
            }
        }
    };

    // Apply timeout if specified
    if let Some(timeout_duration) = timeout {
        tokio::time::timeout(timeout_duration, search_future)
            .await
            .map_err(|_| anyhow!("Search timed out after {:?}", timeout_duration))?;
    } else {
        search_future.await;
    }

    // Check for empty glob
    if files_searched == 0 {
        return Err(anyhow!(
            "No files matched glob pattern '{}'. \
             Check the pattern and ensure matching files exist.",
            config.files
        ));
    }

    // Extract results from sink
    let (count, output_lines, lines_omitted) = sink.into_result();

    Ok(RgSearchResult {
        count,
        output_lines,
        lines_omitted,
        file_counts,
        errors,
        files_searched,
    })
}
```

## Hook Integration

```rust
/// Unified command configuration for execution
pub enum CommandConfig {
    Run(StopCommandConfig),
    Rg(RgCommandConfig),
}

/// Configuration for executing an rg command
pub struct RgCommandConfig {
    pub rg: RgConfig,
    pub message: Option<String>,
    pub action: CommandAction,
    pub show_stdout: bool,
    pub show_stderr: bool,
    pub max_output_lines: Option<u32>,
    pub timeout: Option<Duration>,
}

/// Collect commands from stop config, handling both run and rg
pub fn collect_stop_commands(config: &ConclaudeConfig) -> Result<Vec<CommandConfig>> {
    let mut commands = Vec::new();

    for cmd_config in &config.stop.commands {
        // Validate mutual exclusivity
        match (&cmd_config.run, &cmd_config.rg) {
            (Some(run), None) => {
                // Existing run command handling
                let extracted = extract_bash_commands(run)?;
                for cmd in extracted {
                    commands.push(CommandConfig::Run(StopCommandConfig {
                        command: cmd,
                        message: cmd_config.message.clone(),
                        show_stdout: cmd_config.show_stdout.unwrap_or(false),
                        show_stderr: cmd_config.show_stderr.unwrap_or(false),
                        show_command: cmd_config.show_command.unwrap_or(true),
                        max_output_lines: cmd_config.max_output_lines,
                        timeout: cmd_config.timeout.map(Duration::from_secs),
                    }));
                }
            }
            (None, Some(rg)) => {
                // New rg command
                commands.push(CommandConfig::Rg(RgCommandConfig {
                    rg: rg.clone(),
                    message: cmd_config.message.clone(),
                    action: cmd_config.action,
                    show_stdout: cmd_config.show_stdout.unwrap_or(false),
                    show_stderr: cmd_config.show_stderr.unwrap_or(false),
                    max_output_lines: cmd_config.max_output_lines,
                    timeout: cmd_config.timeout.map(Duration::from_secs),
                }));
            }
            (Some(_), Some(_)) => {
                return Err(anyhow!(
                    "Command cannot have both 'run' and 'rg' fields - they are mutually exclusive"
                ));
            }
            (None, None) => {
                return Err(anyhow!(
                    "Command must have either 'run' or 'rg' field"
                ));
            }
        }
    }

    Ok(commands)
}

/// Execute all stop commands
async fn execute_stop_commands(
    commands: &[CommandConfig],
    config_dir: &Path,
) -> Result<Option<HookResult>> {
    for (idx, cmd) in commands.iter().enumerate() {
        println!("Executing command {}/{}", idx + 1, commands.len());

        match cmd {
            CommandConfig::Run(run_cmd) => {
                // Existing run command execution
                if let Some(result) = execute_run_command(run_cmd, config_dir).await? {
                    return Ok(Some(result));
                }
            }
            CommandConfig::Rg(rg_cmd) => {
                // New rg command execution
                if let Some(result) = execute_rg_hook(rg_cmd, config_dir).await? {
                    return Ok(Some(result));
                }
            }
        }
    }

    Ok(None)
}

/// Execute an rg hook command
async fn execute_rg_hook(
    cmd: &RgCommandConfig,
    config_dir: &Path,
) -> Result<Option<HookResult>> {
    // Execute search
    let result = execute_rg_command(&cmd.rg, config_dir, cmd.timeout).await?;

    // Determine constraint (default: max: 0)
    let constraint = match (&cmd.rg.max, &cmd.rg.min, &cmd.rg.equal) {
        (Some(n), None, None) => Constraint::Max(*n),
        (None, Some(n), None) => Constraint::Min(*n),
        (None, None, Some(n)) => Constraint::Equal(*n),
        (None, None, None) => Constraint::Max(0), // Default
        _ => unreachable!("Validated at config load time"),
    };

    // Evaluate constraint
    match constraint.evaluate(result.count) {
        ConstraintResult::Pass => {
            // Success - continue to next command
            Ok(None)
        }
        ConstraintResult::Fail { message: constraint_msg } => {
            // Build error message
            let error_msg = cmd.message.as_ref()
                .map(|m| format!("{}\n{}", m, constraint_msg))
                .unwrap_or(constraint_msg);

            // Build output
            let mut output = String::new();

            if cmd.show_stdout && !result.output_lines.is_empty() {
                output.push_str(&result.output_lines.join("\n"));
                if result.lines_omitted > 0 {
                    output.push_str(&format!(
                        "\n... ({} lines omitted)",
                        result.lines_omitted
                    ));
                }
            }

            if cmd.show_stderr && !result.errors.is_empty() {
                if !output.is_empty() {
                    output.push_str("\n\n");
                }
                output.push_str("Errors:\n");
                output.push_str(&result.errors.join("\n"));
            }

            match cmd.action {
                CommandAction::Block => {
                    let full_message = if output.is_empty() {
                        error_msg
                    } else {
                        format!("{}\n\n{}", error_msg, output)
                    };

                    Ok(Some(HookResult {
                        blocked: true,
                        message: Some(full_message),
                    }))
                }
                CommandAction::Warn => {
                    // Log warning but don't block
                    eprintln!("Warning: {}", error_msg);
                    if !output.is_empty() {
                        eprintln!("{}", output);
                    }
                    Ok(None)
                }
            }
        }
    }
}
```

## Configuration Validation

```rust
/// Validate RgConfig at load time
fn validate_rg_config(config: &RgConfig, path: &str) -> Result<()> {
    // 1. Validate constraint mutual exclusivity
    let constraint_count = [
        config.max.is_some(),
        config.min.is_some(),
        config.equal.is_some(),
    ].iter().filter(|&&b| b).count();

    if constraint_count > 1 {
        return Err(anyhow!(
            "{}: Only one of 'max', 'min', or 'equal' can be specified",
            path
        ));
    }

    // 2. Validate pattern compiles (unless fixedStrings)
    if !config.fixed_strings {
        let mut builder = RegexMatcherBuilder::new();
        builder
            .case_insensitive(config.ignore_case)
            .case_smart(config.smart_case)
            .unicode(config.unicode);

        if let Err(e) = builder.build(&config.pattern) {
            return Err(anyhow!(
                "{}: Invalid regex pattern '{}': {}",
                path, config.pattern, e
            ));
        }
    }

    // 3. Validate glob pattern
    if let Err(e) = glob::Pattern::new(&config.files) {
        return Err(anyhow!(
            "{}: Invalid glob pattern '{}': {}",
            path, config.files, e
        ));
    }

    // 4. Validate file types
    if !config.types.is_empty() {
        let mut types_builder = ignore::types::TypesBuilder::new();
        types_builder.add_defaults();

        for type_name in &config.types {
            if types_builder.select(type_name).is_err() {
                // Get similar type names for suggestion
                let known_types = get_known_type_names();
                let suggestions = suggest_similar(&type_name, &known_types);

                let mut msg = format!(
                    "{}: Unknown file type '{}'",
                    path, type_name
                );
                if !suggestions.is_empty() {
                    msg.push_str(&format!(
                        ". Did you mean: {}?",
                        suggestions.join(", ")
                    ));
                }
                return Err(anyhow!(msg));
            }
        }
    }

    // 5. Validate numeric ranges
    if config.context > 1000 {
        return Err(anyhow!(
            "{}: context value {} exceeds maximum of 1000",
            path, config.context
        ));
    }

    if let Some(threads) = config.threads {
        if threads == 0 {
            return Err(anyhow!(
                "{}: threads must be at least 1",
                path
            ));
        }
    }

    Ok(())
}

/// Get list of known file type names from ignore crate
fn get_known_type_names() -> Vec<String> {
    let mut builder = ignore::types::TypesBuilder::new();
    builder.add_defaults();
    builder.definitions()
        .iter()
        .map(|d| d.name().to_string())
        .collect()
}
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Binary size increase (~2-3MB) | Acceptable for functionality gained; ripgrep libraries are well-optimized |
| Learning curve for new options | Sensible defaults mean most configs only need pattern + files |
| Performance with large codebases | Use parallel search (threads), respect gitignore by default |
| Complex regex causing slowdowns | Validate regex at config load time; timeout support |
| Pattern mistakes (wrong count mode) | Clear documentation, helpful error messages |

## Migration Plan

1. **Phase 1**: Add dependencies and config structs
2. **Phase 2**: Implement search execution and counting
3. **Phase 3**: Integrate with hook execution pipeline
4. **Phase 4**: Add validation and error handling
5. **Phase 5**: Testing and documentation

### Rollback

The change is purely additive. Existing `run` commands continue to work unchanged. To rollback:
- Remove `rg` field support from config parsing
- Remove ripgrep library dependencies

## Open Questions

1. **Should we expose `encoding` option?** - Ripgrep supports transcoding from various encodings. Most codebases are UTF-8, but some legacy projects might need this.

2. **Should `threads` default to CPU count or 1?** - Parallel search is faster but may impact system responsiveness during hook execution.

3. **Should we support multiple patterns?** - Ripgrep supports `--regexp` multiple times. Could add `patterns: [...]` field.

4. **Should we add `exclude` patterns?** - Sometimes easier to say "all files except X" rather than complex globs.

# Design: Tree-sitter Hook Command Implementation

## Overview

This document details the implementation architecture for the `ts` hook command type, which enables structural code analysis using tree-sitter queries.

## Module Structure

```
src/
  config.rs          # Extended with TsConfig, TsCommand structs
  hooks.rs           # Extended with execute_ts_command()
  treesitter/        # NEW: Tree-sitter integration module
    mod.rs           # Module exports
    languages.rs     # Language registry and detection
    query.rs         # Query compilation and validation
    searcher.rs      # File traversal and query execution
    sink.rs          # Match collection and counting
    output.rs        # Output formatting
```

## Core Data Structures

### Configuration (config.rs)

```rust
/// Tree-sitter query configuration
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    /// Tree-sitter S-expression query
    pub query: String,

    /// Glob pattern for files to search
    pub files: String,

    /// Which capture to count (default: first in query)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture: Option<String>,

    /// Override auto-detected language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    // Constraint fields (mutually exclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equal: Option<u64>,

    // File walking options (shared with RgConfig)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub follow_links: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_filesize: Option<u64>,
    #[serde(default = "default_true")]
    pub git_ignore: bool,
    #[serde(default = "default_true")]
    pub ignore: bool,
    #[serde(default = "default_true")]
    pub parents: bool,
    #[serde(default)]
    pub same_file_system: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threads: Option<usize>,
}

/// Stop command now has three mutually exclusive variants
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StopCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rg: Option<RgConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<TsConfig>,  // NEW

    // Common fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default)]
    pub action: CommandAction,
    #[serde(default)]
    pub show_stdout: bool,
    #[serde(default)]
    pub show_stderr: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_lines: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}
```

### Language Registry (treesitter/languages.rs)

```rust
use tree_sitter::Language;
use std::collections::HashMap;

/// Registry of bundled tree-sitter languages
pub struct LanguageRegistry {
    /// Map from language name to Language
    languages: HashMap<&'static str, Language>,
    /// Map from file extension to language name
    extensions: HashMap<&'static str, &'static str>,
}

impl LanguageRegistry {
    /// Create registry with all bundled languages
    pub fn new() -> Self {
        let mut registry = Self {
            languages: HashMap::new(),
            extensions: HashMap::new(),
        };

        // Register all bundled languages
        registry.register("rust", tree_sitter_rust::LANGUAGE.into(), &["rs"]);
        registry.register("javascript", tree_sitter_javascript::LANGUAGE.into(),
                         &["js", "mjs", "cjs"]);
        registry.register("typescript", tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                         &["ts"]);
        registry.register("tsx", tree_sitter_typescript::LANGUAGE_TSX.into(),
                         &["tsx"]);
        registry.register("python", tree_sitter_python::LANGUAGE.into(), &["py"]);
        registry.register("go", tree_sitter_go::LANGUAGE.into(), &["go"]);
        registry.register("java", tree_sitter_java::LANGUAGE.into(), &["java"]);
        registry.register("c", tree_sitter_c::LANGUAGE.into(), &["c", "h"]);
        registry.register("cpp", tree_sitter_cpp::LANGUAGE.into(),
                         &["cpp", "cc", "cxx", "hpp", "hxx"]);
        // ... additional languages ...

        registry
    }

    fn register(&mut self, name: &'static str, lang: Language, exts: &[&'static str]) {
        self.languages.insert(name, lang);
        for ext in exts {
            self.extensions.insert(ext, name);
        }
    }

    /// Get language by name
    pub fn get_language(&self, name: &str) -> Option<Language> {
        self.languages.get(name).copied()
    }

    /// Detect language from file extension
    pub fn detect_language(&self, path: &Path) -> Option<(&'static str, Language)> {
        let ext = path.extension()?.to_str()?;
        let name = self.extensions.get(ext)?;
        let lang = self.languages.get(name)?;
        Some((name, *lang))
    }

    /// List all supported language names
    pub fn supported_languages(&self) -> Vec<&'static str> {
        self.languages.keys().copied().collect()
    }
}

/// Global singleton (lazy_static or once_cell)
pub fn language_registry() -> &'static LanguageRegistry {
    static REGISTRY: OnceLock<LanguageRegistry> = OnceLock::new();
    REGISTRY.get_or_init(LanguageRegistry::new)
}
```

### Compiled Query (treesitter/query.rs)

```rust
use tree_sitter::{Query, Language, QueryError};

/// A validated and compiled tree-sitter query
pub struct CompiledQuery {
    /// The compiled query
    query: Query,
    /// Name of capture to count
    target_capture: String,
    /// Index of target capture in query
    target_capture_index: u32,
    /// All capture names in query
    capture_names: Vec<String>,
}

impl CompiledQuery {
    /// Compile a query for a given language
    pub fn new(
        language: Language,
        query_source: &str,
        target_capture: Option<&str>,
    ) -> Result<Self, QueryCompileError> {
        // Compile the query
        let query = Query::new(&language, query_source)
            .map_err(|e| QueryCompileError::InvalidSyntax(e))?;

        // Extract capture names
        let capture_names: Vec<String> = query
            .capture_names()
            .iter()
            .map(|s| format!("@{}", s))
            .collect();

        if capture_names.is_empty() {
            return Err(QueryCompileError::NoCaptures);
        }

        // Determine target capture
        let target_capture = match target_capture {
            Some(name) => {
                if !capture_names.contains(&name.to_string()) {
                    return Err(QueryCompileError::UnknownCapture {
                        requested: name.to_string(),
                        available: capture_names,
                    });
                }
                name.to_string()
            }
            None => capture_names[0].clone(),
        };

        // Get capture index (strip @ prefix for tree-sitter API)
        let capture_name_bare = target_capture.trim_start_matches('@');
        let target_capture_index = query
            .capture_index_for_name(capture_name_bare)
            .ok_or_else(|| QueryCompileError::UnknownCapture {
                requested: target_capture.clone(),
                available: capture_names.clone(),
            })?;

        Ok(Self {
            query,
            target_capture,
            target_capture_index,
            capture_names,
        })
    }

    /// Get the underlying query
    pub fn query(&self) -> &Query {
        &self.query
    }

    /// Get target capture index
    pub fn target_capture_index(&self) -> u32 {
        self.target_capture_index
    }

    /// Get target capture name
    pub fn target_capture_name(&self) -> &str {
        &self.target_capture
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryCompileError {
    #[error("Invalid query syntax: {0}")]
    InvalidSyntax(QueryError),

    #[error("Query has no captures - at least one @capture is required")]
    NoCaptures,

    #[error("Unknown capture {requested}, available: {}", available.join(", "))]
    UnknownCapture {
        requested: String,
        available: Vec<String>,
    },
}
```

### Match Collector (treesitter/sink.rs)

```rust
use tree_sitter::{Node, QueryCursor, QueryMatch};

/// Information about a single match
#[derive(Debug, Clone)]
pub struct MatchInfo {
    pub file: PathBuf,
    pub line: usize,      // 1-based
    pub column: usize,    // 1-based
    pub node_kind: String,
    pub text: String,
}

impl MatchInfo {
    /// Format as detailed output line
    pub fn format_detailed(&self, max_text_len: usize) -> String {
        let text = self.truncate_text(max_text_len);
        format!("{}:{}:{} [{}]: {}",
            self.file.display(),
            self.line,
            self.column,
            self.node_kind,
            text
        )
    }

    fn truncate_text(&self, max_len: usize) -> String {
        // Take first line only
        let first_line = self.text.lines().next().unwrap_or(&self.text);

        if first_line.len() > max_len {
            format!("{}...", &first_line[..max_len])
        } else if self.text.lines().count() > 1 {
            format!("{}...", first_line)
        } else {
            first_line.to_string()
        }
    }
}

/// Collects matches from query execution
pub struct MatchCollector {
    matches: Vec<MatchInfo>,
    count: u64,
    target_capture_index: u32,
}

impl MatchCollector {
    pub fn new(target_capture_index: u32) -> Self {
        Self {
            matches: Vec::new(),
            count: 0,
            target_capture_index,
        }
    }

    /// Process matches from a file
    pub fn process_file(
        &mut self,
        file: &Path,
        source: &[u8],
        tree: &tree_sitter::Tree,
        query: &Query,
    ) {
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), source);

        for query_match in matches {
            for capture in query_match.captures {
                if capture.index == self.target_capture_index {
                    self.count += 1;

                    let node = capture.node;
                    let start = node.start_position();

                    self.matches.push(MatchInfo {
                        file: file.to_path_buf(),
                        line: start.row + 1,  // Convert to 1-based
                        column: start.column + 1,
                        node_kind: node.kind().to_string(),
                        text: node.utf8_text(source).unwrap_or("<binary>").to_string(),
                    });
                }
            }
        }
    }

    /// Get total count
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Get matches (for output)
    pub fn matches(&self) -> &[MatchInfo] {
        &self.matches
    }
}
```

### Tree-sitter Searcher (treesitter/searcher.rs)

```rust
use ignore::{WalkBuilder, WalkState};
use tree_sitter::Parser;
use std::sync::{Arc, Mutex};

/// Configuration for tree-sitter search
pub struct TsSearchConfig {
    pub query: CompiledQuery,
    pub language: Option<String>,  // Override language detection
    pub walk_config: WalkConfig,   // Shared with RgConfig
}

/// Execute tree-sitter search across files
pub struct TsSearcher {
    config: TsSearchConfig,
    registry: &'static LanguageRegistry,
}

impl TsSearcher {
    pub fn new(config: TsSearchConfig) -> Self {
        Self {
            config,
            registry: language_registry(),
        }
    }

    /// Execute search and return results
    pub fn search(&self, base_path: &Path) -> Result<TsSearchResult, TsSearchError> {
        let collector = Arc::new(Mutex::new(MatchCollector::new(
            self.config.query.target_capture_index()
        )));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let files_matched = Arc::new(AtomicU64::new(0));

        // Build file walker (same as rg command)
        let walker = self.build_walker(base_path)?;

        // Process files in parallel
        walker.build_parallel().run(|| {
            let collector = Arc::clone(&collector);
            let errors = Arc::clone(&errors);
            let files_matched = Arc::clone(&files_matched);
            let query = &self.config.query;
            let override_lang = &self.config.language;
            let registry = self.registry;

            // Each thread gets its own parser
            let mut parser = Parser::new();

            Box::new(move |entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        errors.lock().unwrap().push(format!("Walk error: {}", e));
                        return WalkState::Continue;
                    }
                };

                // Skip directories
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    return WalkState::Continue;
                }

                let path = entry.path();

                // Determine language
                let language = if let Some(lang_name) = override_lang {
                    registry.get_language(lang_name)
                } else {
                    registry.detect_language(path).map(|(_, lang)| lang)
                };

                let language = match language {
                    Some(l) => l,
                    None => {
                        // Skip files with unknown language
                        return WalkState::Continue;
                    }
                };

                // Set parser language
                if parser.set_language(&language).is_err() {
                    errors.lock().unwrap().push(
                        format!("Failed to set language for {}", path.display())
                    );
                    return WalkState::Continue;
                }

                // Read file
                let source = match std::fs::read(path) {
                    Ok(s) => s,
                    Err(e) => {
                        errors.lock().unwrap().push(
                            format!("Read error {}: {}", path.display(), e)
                        );
                        return WalkState::Continue;
                    }
                };

                // Parse file
                let tree = match parser.parse(&source, None) {
                    Some(t) => t,
                    None => {
                        errors.lock().unwrap().push(
                            format!("Parse failed: {}", path.display())
                        );
                        return WalkState::Continue;
                    }
                };

                files_matched.fetch_add(1, Ordering::Relaxed);

                // Execute query and collect matches
                collector.lock().unwrap().process_file(
                    path,
                    &source,
                    &tree,
                    query.query(),
                );

                WalkState::Continue
            })
        });

        let files_matched = files_matched.load(Ordering::Relaxed);

        // Check for empty glob
        if files_matched == 0 {
            return Err(TsSearchError::NoFilesMatched);
        }

        let collector = Arc::try_unwrap(collector)
            .expect("All threads finished")
            .into_inner()
            .unwrap();
        let errors = Arc::try_unwrap(errors)
            .expect("All threads finished")
            .into_inner()
            .unwrap();

        Ok(TsSearchResult {
            count: collector.count(),
            matches: collector.matches().to_vec(),
            files_searched: files_matched,
            errors,
        })
    }

    fn build_walker(&self, base_path: &Path) -> Result<WalkBuilder, TsSearchError> {
        let mut builder = WalkBuilder::new(base_path);

        // Apply glob filter
        builder.types(self.build_glob_types()?);

        // Apply walk options (same as rg)
        let cfg = &self.config.walk_config;
        if let Some(depth) = cfg.max_depth {
            builder.max_depth(Some(depth));
        }
        builder.hidden(!cfg.hidden);
        builder.follow_links(cfg.follow_links);
        builder.git_ignore(cfg.git_ignore);
        builder.ignore(cfg.ignore);
        builder.parents(cfg.parents);
        builder.same_file_system(cfg.same_file_system);
        if let Some(threads) = cfg.threads {
            builder.threads(threads);
        }

        Ok(builder)
    }
}

/// Result of tree-sitter search
pub struct TsSearchResult {
    pub count: u64,
    pub matches: Vec<MatchInfo>,
    pub files_searched: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TsSearchError {
    #[error("No files matched the glob pattern")]
    NoFilesMatched,

    #[error("Glob pattern error: {0}")]
    GlobError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Data Flow

```
                         Configuration Load
                               │
                               ▼
                    ┌──────────────────────┐
                    │   Parse TsConfig     │
                    │   from YAML/JSON     │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │  Validate Config     │
                    │  - Query syntax      │
                    │  - Capture exists    │
                    │  - Language valid    │
                    │  - Constraints       │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │  Compile Query       │
                    │  (tree_sitter::Query)│
                    └──────────┬───────────┘
                               │
                               │
                         Hook Execution
                               │
                               ▼
                    ┌──────────────────────┐
                    │  Build File Walker   │
                    │  (ignore::WalkBuilder)│
                    └──────────┬───────────┘
                               │
                               ▼
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
        ┌──────────┐    ┌──────────┐    ┌──────────┐
        │ Thread 1 │    │ Thread 2 │    │ Thread N │
        │          │    │          │    │          │
        │ Parser   │    │ Parser   │    │ Parser   │
        │ per file │    │ per file │    │ per file │
        └────┬─────┘    └────┬─────┘    └────┬─────┘
             │               │               │
             └───────────────┼───────────────┘
                             │
                             ▼
                    ┌──────────────────────┐
                    │   Match Collector    │
                    │   (Arc<Mutex<...>>)  │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │  Evaluate Constraint │
                    │  max/min/equal       │
                    └──────────┬───────────┘
                               │
                  ┌────────────┴────────────┐
                  │                         │
                  ▼                         ▼
         ┌──────────────┐         ┌──────────────┐
         │   Pass       │         │   Fail       │
         │ (count ok)   │         │ (constraint  │
         └──────────────┘         │  violated)   │
                                  └──────┬───────┘
                                         │
                                         ▼
                                ┌──────────────────┐
                                │  Format Output   │
                                │  file:line [kind]│
                                └──────────────────┘
```

## Integration with Existing Code

### hooks.rs Changes

```rust
// In collect_stop_commands()
pub fn collect_stop_commands(config: &Config) -> Vec<StopCommandConfig> {
    let mut commands = Vec::new();

    if let Some(stop_config) = &config.hooks.stop {
        for cmd in &stop_config.commands {
            if let Some(run) = &cmd.run {
                commands.push(StopCommandConfig::Run(RunCommandConfig { ... }));
            } else if let Some(rg) = &cmd.rg {
                commands.push(StopCommandConfig::Rg(RgCommandConfig { ... }));
            } else if let Some(ts) = &cmd.ts {
                commands.push(StopCommandConfig::Ts(TsCommandConfig::from(ts, cmd)));
            }
        }
    }

    commands
}

// New enum variant
pub enum StopCommandConfig {
    Run(RunCommandConfig),
    Rg(RgCommandConfig),
    Ts(TsCommandConfig),  // NEW
}

// Execution dispatch
pub async fn execute_stop_command(cmd: &StopCommandConfig) -> CommandResult {
    match cmd {
        StopCommandConfig::Run(c) => execute_run_command(c).await,
        StopCommandConfig::Rg(c) => execute_rg_command(c).await,
        StopCommandConfig::Ts(c) => execute_ts_command(c).await,  // NEW
    }
}
```

### execute_ts_command Implementation

```rust
pub async fn execute_ts_command(config: &TsCommandConfig) -> CommandResult {
    // Build searcher
    let searcher = TsSearcher::new(TsSearchConfig {
        query: config.compiled_query.clone(),
        language: config.language.clone(),
        walk_config: config.walk_config.clone(),
    });

    // Execute with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(config.timeout.unwrap_or(120)),
        tokio::task::spawn_blocking(move || {
            searcher.search(&config.base_path)
        })
    ).await;

    let result = match result {
        Ok(Ok(Ok(r))) => r,
        Ok(Ok(Err(e))) => return CommandResult::error(e.to_string()),
        Ok(Err(e)) => return CommandResult::error(format!("Search panicked: {}", e)),
        Err(_) => return CommandResult::error("Search timed out"),
    };

    // Evaluate constraint
    let (passed, constraint_msg) = evaluate_constraint(
        result.count,
        config.max,
        config.min,
        config.equal,
        &config.compiled_query.target_capture_name(),
    );

    if passed {
        return CommandResult::success();
    }

    // Build error message
    let mut message = config.message.clone()
        .unwrap_or_else(|| constraint_msg.clone());

    if config.show_stdout && !result.matches.is_empty() {
        let output = format_matches(
            &result.matches,
            config.max_output_lines.unwrap_or(usize::MAX),
        );
        message = format!("{}\n\n{}", message, output);
    }

    if config.show_stderr && !result.errors.is_empty() {
        message = format!("{}\n\nWarnings:\n{}", message, result.errors.join("\n"));
    }

    match config.action {
        CommandAction::Block => CommandResult::blocked(message),
        CommandAction::Warn => {
            eprintln!("Warning: {}", message);
            CommandResult::success()
        }
    }
}

fn evaluate_constraint(
    count: u64,
    max: Option<u64>,
    min: Option<u64>,
    equal: Option<u64>,
    capture_name: &str,
) -> (bool, String) {
    if let Some(max_val) = max {
        let passed = count <= max_val;
        let msg = format!(
            "Found {} captures of {}, maximum allowed is {}",
            count, capture_name, max_val
        );
        return (passed, msg);
    }

    if let Some(min_val) = min {
        let passed = count >= min_val;
        let msg = format!(
            "Found {} captures of {}, minimum required is {}",
            count, capture_name, min_val
        );
        return (passed, msg);
    }

    if let Some(equal_val) = equal {
        let passed = count == equal_val;
        let msg = format!(
            "Found {} captures of {}, expected exactly {}",
            count, capture_name, equal_val
        );
        return (passed, msg);
    }

    // Default: max: 0
    let passed = count == 0;
    let msg = format!(
        "Found {} captures of {}, maximum allowed is 0",
        count, capture_name
    );
    (passed, msg)
}
```

## Performance Considerations

### Parser Reuse

Each thread maintains its own `Parser` instance to avoid lock contention:

```rust
// BAD: Shared parser with lock
let parser = Arc::new(Mutex::new(Parser::new()));

// GOOD: Per-thread parser
walker.build_parallel().run(|| {
    let mut parser = Parser::new();  // Each thread gets own parser
    Box::new(move |entry| { ... })
});
```

### Query Compilation

Queries are compiled once and shared (Query is Send + Sync):

```rust
// Compile once during config validation
let query = CompiledQuery::new(language, query_source, capture)?;

// Share across threads (Query is internally immutable)
let query = Arc::new(query);
```

### Memory Mapping vs Reading

For large files, consider memory mapping (tree-sitter supports it):

```rust
// For files > 1MB, use mmap
if file_size > 1_000_000 {
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    parser.parse(&mmap[..], None)
} else {
    let source = std::fs::read(path)?;
    parser.parse(&source, None)
}
```

### Early Termination

For `max: N` constraints, stop after finding N+1 matches:

```rust
impl MatchCollector {
    pub fn should_stop(&self, max_constraint: Option<u64>) -> bool {
        if let Some(max) = max_constraint {
            self.count > max
        } else {
            false
        }
    }
}
```

## Error Handling Strategy

| Error Type | Handling |
|------------|----------|
| Invalid query syntax | Fail at config load time with helpful error |
| Unknown capture | Fail at config load time, list available captures |
| Unknown language | Fail at config load time, list supported languages |
| File read error | Skip file, log to errors (shown with showStderr) |
| Parse error | Skip file (tree-sitter is error-tolerant, but edge cases) |
| Empty glob | Fail with clear error message |
| Timeout | Return timeout error, abort search |

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_query_compilation() {
    let lang = tree_sitter_rust::LANGUAGE.into();
    let query = CompiledQuery::new(
        lang,
        "(function_item name: (identifier) @name) @fn",
        Some("@fn"),
    );
    assert!(query.is_ok());
    assert_eq!(query.unwrap().target_capture_name(), "@fn");
}

#[test]
fn test_unknown_capture() {
    let lang = tree_sitter_rust::LANGUAGE.into();
    let query = CompiledQuery::new(
        lang,
        "(function_item) @fn",
        Some("@unknown"),
    );
    assert!(matches!(query, Err(QueryCompileError::UnknownCapture { .. })));
}
```

### Integration Tests

```rust
#[test]
fn test_rust_unsafe_detection() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("lib.rs"), r#"
        fn safe() {}
        fn danger() { unsafe { std::ptr::null::<i32>().read() } }
    "#).unwrap();

    let config = TsConfig {
        query: "(unsafe_block) @unsafe".to_string(),
        files: "**/*.rs".to_string(),
        max: Some(0),
        ..Default::default()
    };

    let result = execute_ts_search(&config, dir.path());
    assert_eq!(result.count, 1);
    assert!(!result.matches.is_empty());
}
```

## Dependency Versions

```toml
[dependencies]
tree-sitter = "0.24"

# Individual language crates (or use rs-tree-sitter-languages)
tree-sitter-rust = "0.23"
tree-sitter-javascript = "0.23"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-go = "0.23"
tree-sitter-java = "0.23"
tree-sitter-c = "0.23"
tree-sitter-cpp = "0.23"
tree-sitter-html = "0.23"
tree-sitter-css = "0.23"
tree-sitter-json = "0.24"
tree-sitter-yaml = "0.6"
tree-sitter-toml = "0.6"
tree-sitter-markdown = "0.3"
# ... additional languages
```

## Future Considerations

1. **Dynamic grammar loading** - Load grammars from .so files at runtime to reduce binary size
2. **Query caching** - Cache compiled queries in a global registry
3. **Incremental parsing** - For watch mode, reuse trees with `InputEdit`
4. **Pattern syntax** - Add ast-grep style `fn $NAME() { $$$BODY }` patterns
5. **Query files** - Support `queryFile: "queries/no-unsafe.scm"` for complex queries

//! Ripgrep-based search functionality for hook commands.
//!
//! This module provides file content searching using ripgrep's library crates,
//! offering a declarative alternative to shell commands for pattern matching.

use crate::config::{CountMode, RgConfig};
use anyhow::{Context, Result};
use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use ignore::overrides::OverrideBuilder;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

/// Result of a ripgrep search operation
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

/// Result of constraint evaluation
#[derive(Debug)]
pub enum ConstraintResult {
    Pass,
    Fail { message: String },
}

impl Constraint {
    /// Create a constraint from RgConfig, defaulting to Max(0) if none specified
    pub fn from_config(config: &RgConfig) -> Self {
        match (config.max, config.min, config.equal) {
            (Some(n), None, None) => Constraint::Max(n),
            (None, Some(n), None) => Constraint::Min(n),
            (None, None, Some(n)) => Constraint::Equal(n),
            _ => Constraint::Max(0), // Default: any match = fail
        }
    }

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

/// Sink implementation for counting matches and collecting output
struct CountingSink<'a, M: Matcher> {
    count: u64,
    count_mode: CountMode,
    output: Vec<String>,
    max_lines: Option<usize>,
    lines_omitted: usize,
    current_file: PathBuf,
    matcher: &'a M,
}

impl<'a, M: Matcher> CountingSink<'a, M> {
    fn new(count_mode: CountMode, max_lines: Option<usize>, matcher: &'a M) -> Self {
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

    fn set_file(&mut self, path: PathBuf) {
        self.current_file = path;
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

    fn into_result(self) -> (u64, Vec<String>, usize) {
        (self.count, self.output, self.lines_omitted)
    }
}

impl<'a, M: Matcher> Sink for CountingSink<'a, M> {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &grep_searcher::Searcher,
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
                let _ = self.matcher.find_iter(line, |_| {
                    count += 1;
                    true
                });
                self.count += count.max(1); // At least 1 if we got a match
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
}

/// Execute a ripgrep search based on the provided configuration
///
/// # Arguments
///
/// * `config` - The RgConfig specifying search parameters
/// * `base_path` - The base directory for the search
/// * `max_output_lines` - Optional limit on output lines
///
/// # Errors
///
/// Returns an error if the regex pattern is invalid, the glob pattern is invalid,
/// or no files match the glob pattern.
pub fn execute_rg_search(
    config: &RgConfig,
    base_path: &Path,
    max_output_lines: Option<u32>,
) -> Result<RgSearchResult> {
    // Build the regex matcher
    let mut builder = RegexMatcherBuilder::new();
    builder
        .case_insensitive(config.ignore_case)
        .case_smart(config.smart_case)
        .unicode(config.unicode)
        .word(config.word)
        .multi_line(config.multi_line)
        .dot_matches_new_line(config.dot_matches_new_line);

    if config.fixed_strings {
        builder.fixed_strings(true);
    }

    let matcher = builder
        .build(&config.pattern)
        .with_context(|| format!("Invalid regex pattern: {}", config.pattern))?;

    // Build the file walker
    let mut walk_builder = WalkBuilder::new(base_path);
    walk_builder
        .hidden(!config.hidden)
        .git_ignore(config.git_ignore)
        .git_global(config.git_ignore)
        .git_exclude(config.git_ignore)
        .ignore(config.rg_ignore)
        .parents(config.parents)
        .follow_links(config.follow_links)
        .same_file_system(config.same_file_system);

    if let Some(depth) = config.max_depth {
        walk_builder.max_depth(Some(depth));
    }
    if let Some(threads) = config.threads {
        walk_builder.threads(threads);
    }

    // Add file type filtering if specified
    if !config.types.is_empty() {
        let mut types_builder = TypesBuilder::new();
        types_builder.add_defaults();
        for type_name in &config.types {
            types_builder.select(type_name);
        }
        let types = types_builder
            .build()
            .with_context(|| format!("Failed to build file types. Check that all type names are valid: {:?}", config.types))?;
        walk_builder.types(types);
    }

    // Add glob override for files pattern
    let mut overrides = OverrideBuilder::new(base_path);
    overrides
        .add(&config.files)
        .with_context(|| format!("Invalid glob pattern: {}", config.files))?;
    walk_builder.overrides(overrides.build().context("Failed to build glob override")?);

    // Build the searcher
    let mut searcher_builder = SearcherBuilder::new();
    searcher_builder
        .line_number(true)
        .binary_detection(BinaryDetection::quit(0x00))
        .before_context(config.context)
        .after_context(config.context);

    if config.invert_match {
        searcher_builder.invert_match(true);
    }

    let mut searcher = searcher_builder.build();

    // Execute search
    let max_lines = max_output_lines.map(|n| n as usize);
    let mut sink = CountingSink::new(config.count_mode, max_lines, &matcher);
    let mut files_searched = 0u64;
    let mut file_counts: HashMap<PathBuf, u64> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();

    for entry in walk_builder.build() {
        match entry {
            Ok(entry) => {
                // Skip directories
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }

                let path = entry.path().to_path_buf();

                // Skip files over max filesize if specified
                if let Some(max_size) = config.max_filesize {
                    if let Ok(metadata) = path.metadata() {
                        if metadata.len() > max_size {
                            continue;
                        }
                    }
                }

                files_searched += 1;
                sink.set_file(path.clone());

                let count_before = sink.count;
                match searcher.search_path(&matcher, &path, &mut sink) {
                    Ok(()) => {
                        let matches_in_file = sink.count - count_before;
                        if matches_in_file > 0 {
                            file_counts.insert(path, matches_in_file);
                        }
                    }
                    Err(e) => {
                        // Non-fatal errors are collected
                        errors.push(format!("{}: {}", path.display(), e));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Walk error: {}", e));
            }
        }
    }

    // Check for empty glob
    if files_searched == 0 {
        return Err(anyhow::anyhow!(
            "No files matched glob pattern '{}'. Check the pattern and ensure matching files exist.",
            config.files
        ));
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Helper to create test files
    fn create_test_files(temp_dir: &TempDir) -> PathBuf {
        let dir = temp_dir.path();

        // Create a Rust file with some patterns
        // Note: no leading newline to avoid extra match
        fs::write(
            dir.join("test.rs"),
            r#"fn main() {
    // TODO: Fix this
    println!("Hello");
    // TODO: Another todo
}
"#,
        )
        .unwrap();

        // Create another file
        fs::write(
            dir.join("lib.rs"),
            r#"// This file has no items
fn lib_fn() {}
"#,
        )
        .unwrap();

        dir.to_path_buf()
    }

    // ========== Constraint Tests ==========

    #[test]
    fn test_constraint_max_pass() {
        let constraint = Constraint::Max(5);
        assert!(matches!(constraint.evaluate(3), ConstraintResult::Pass));
        assert!(matches!(constraint.evaluate(5), ConstraintResult::Pass));
    }

    #[test]
    fn test_constraint_max_fail() {
        let constraint = Constraint::Max(5);
        match constraint.evaluate(6) {
            ConstraintResult::Fail { message } => {
                assert!(message.contains("6 matches"));
                assert!(message.contains("maximum allowed is 5"));
            }
            ConstraintResult::Pass => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_constraint_min_pass() {
        let constraint = Constraint::Min(3);
        assert!(matches!(constraint.evaluate(3), ConstraintResult::Pass));
        assert!(matches!(constraint.evaluate(5), ConstraintResult::Pass));
    }

    #[test]
    fn test_constraint_min_fail() {
        let constraint = Constraint::Min(3);
        match constraint.evaluate(2) {
            ConstraintResult::Fail { message } => {
                assert!(message.contains("2 matches"));
                assert!(message.contains("minimum required is 3"));
            }
            ConstraintResult::Pass => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_constraint_equal_pass() {
        let constraint = Constraint::Equal(5);
        assert!(matches!(constraint.evaluate(5), ConstraintResult::Pass));
    }

    #[test]
    fn test_constraint_equal_fail() {
        let constraint = Constraint::Equal(5);
        match constraint.evaluate(3) {
            ConstraintResult::Fail { message } => {
                assert!(message.contains("3 matches"));
                assert!(message.contains("expected exactly 5"));
            }
            ConstraintResult::Pass => panic!("Expected failure"),
        }
    }

    // ========== Search Functionality Tests ==========

    #[test]
    fn test_search_finds_matches() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "TODO".to_string(),
            files: "**/*.rs".to_string(),
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, None).unwrap();
        assert_eq!(result.count, 2); // Two TODOs
        assert_eq!(result.files_searched, 2);
        assert_eq!(result.file_counts.len(), 1); // Only test.rs has matches
    }

    #[test]
    fn test_search_respects_max_constraint() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "TODO".to_string(),
            files: "**/*.rs".to_string(),
            max: Some(1),
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, None).unwrap();
        let constraint = Constraint::from_config(&config);
        match constraint.evaluate(result.count) {
            ConstraintResult::Fail { message } => {
                assert!(message.contains("2 matches"));
                assert!(message.contains("maximum allowed is 1"));
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_search_respects_min_constraint() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "NONEXISTENT".to_string(),
            files: "**/*.rs".to_string(),
            min: Some(1),
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, None).unwrap();
        let constraint = Constraint::from_config(&config);
        match constraint.evaluate(result.count) {
            ConstraintResult::Fail { message } => {
                assert!(message.contains("0 matches"));
                assert!(message.contains("minimum required is 1"));
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "todo".to_string(), // lowercase
            files: "**/*.rs".to_string(),
            ignore_case: true,
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, None).unwrap();
        assert_eq!(result.count, 2); // Should find TODO with case insensitive
    }

    #[test]
    fn test_search_max_output_lines() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "TODO".to_string(),
            files: "**/*.rs".to_string(),
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, Some(1)).unwrap();
        assert_eq!(result.output_lines.len(), 1);
        assert_eq!(result.lines_omitted, 1);
    }

    #[test]
    fn test_search_empty_glob_error() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let config = RgConfig {
            pattern: "TODO".to_string(),
            files: "**/*.nonexistent".to_string(),
            ..Default::default()
        };

        let result = execute_rg_search(&config, base_path, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No files matched"));
    }

    #[test]
    fn test_search_with_types_filter() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "TODO".to_string(),
            files: "*".to_string(),
            types: vec!["rust".to_string()],
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, None).unwrap();
        assert_eq!(result.count, 2);
    }

    #[test]
    fn test_search_equal_constraint_pass() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "TODO".to_string(),
            files: "**/*.rs".to_string(),
            equal: Some(2),
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, None).unwrap();
        let constraint = Constraint::from_config(&config);
        match constraint.evaluate(result.count) {
            ConstraintResult::Pass => {} // Expected
            ConstraintResult::Fail { message } => {
                panic!("Expected pass, got failure: {}", message)
            }
        }
    }

    #[test]
    fn test_search_fixed_strings() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        fs::write(
            dir.join("test.txt"),
            r#"This contains [brackets] literally
Not a regex pattern
"#,
        )
        .unwrap();

        let config = RgConfig {
            pattern: "[brackets]".to_string(), // Would be invalid regex
            files: "**/*.txt".to_string(),
            fixed_strings: true,
            ..Default::default()
        };

        let result = execute_rg_search(&config, dir, None).unwrap();
        assert_eq!(result.count, 1); // Should find the literal string
    }

    #[test]
    fn test_search_context_lines() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        fs::write(
            dir.join("context.txt"),
            r#"line1
line2
MATCH
line4
line5"#,
        )
        .unwrap();

        let config = RgConfig {
            pattern: "MATCH".to_string(),
            files: "**/*.txt".to_string(),
            context: 1,
            ..Default::default()
        };

        let result = execute_rg_search(&config, dir, None).unwrap();
        assert_eq!(result.count, 1);
        // Context lines are included in output but not in count
    }

    #[test]
    fn test_constraint_from_config_default() {
        let config = RgConfig {
            pattern: "test".to_string(),
            files: "*.rs".to_string(),
            ..Default::default()
        };

        let constraint = Constraint::from_config(&config);
        match constraint {
            Constraint::Max(n) => assert_eq!(n, 0),
            _ => panic!("Expected Max(0) for default constraint"),
        }
    }

    #[test]
    fn test_constraint_from_config_max() {
        let config = RgConfig {
            pattern: "test".to_string(),
            files: "*.rs".to_string(),
            max: Some(10),
            ..Default::default()
        };

        let constraint = Constraint::from_config(&config);
        match constraint {
            Constraint::Max(n) => assert_eq!(n, 10),
            _ => panic!("Expected Max(10)"),
        }
    }

    #[test]
    fn test_constraint_from_config_min() {
        let config = RgConfig {
            pattern: "test".to_string(),
            files: "*.rs".to_string(),
            min: Some(5),
            ..Default::default()
        };

        let constraint = Constraint::from_config(&config);
        match constraint {
            Constraint::Min(n) => assert_eq!(n, 5),
            _ => panic!("Expected Min(5)"),
        }
    }

    #[test]
    fn test_constraint_from_config_equal() {
        let config = RgConfig {
            pattern: "test".to_string(),
            files: "*.rs".to_string(),
            equal: Some(3),
            ..Default::default()
        };

        let constraint = Constraint::from_config(&config);
        match constraint {
            Constraint::Equal(n) => assert_eq!(n, 3),
            _ => panic!("Expected Equal(3)"),
        }
    }

    #[test]
    fn test_search_result_file_counts() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_files(&temp_dir);

        let config = RgConfig {
            pattern: "TODO".to_string(),
            files: "**/*.rs".to_string(),
            ..Default::default()
        };

        let result = execute_rg_search(&config, &base_path, None).unwrap();

        // Check file_counts HashMap
        assert_eq!(result.file_counts.len(), 1);
        let test_rs_path = base_path.join("test.rs");
        assert_eq!(result.file_counts.get(&test_rs_path), Some(&2));
    }
}

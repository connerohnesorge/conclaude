//! Git-ignore pattern matching functionality.
//!
//! This module provides utilities for checking if files are ignored by `.gitignore` patterns.
//! It wraps the `ignore` crate's gitignore functionality with a simple API focused on
//! single-file path checking.
//!
//! Supports:
//! - Global git excludes (`~/.config/git/ignore`, `~/.gitignore`, `core.excludesfile`)
//! - Repository-specific excludes (`.git/info/exclude`)
//! - Root and nested `.gitignore` files with proper precedence
//!
//! # Limitations
//!
//! - Both new file creation (Write) and modification (Edit) of git-ignored files are blocked.
//!   This is intentional - if a file should be ignored, Claude shouldn't create or modify it.
//!
//! # Examples
//!
//! ```rust
//! use conclaude::gitignore::is_path_git_ignored;
//! use std::path::Path;
//!
//! # fn example() -> anyhow::Result<()> {
//! let repo_root = Path::new("/path/to/repo");
//! let (is_ignored, pattern) = is_path_git_ignored(
//!     Path::new("node_modules/foo.js"),
//!     repo_root
//! )?;
//!
//! if is_ignored {
//!     println!("File is ignored, matched pattern: {:?}", pattern);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the global git excludes file.
///
/// Checks in order:
/// 1. `git config --global core.excludesfile`
/// 2. `~/.config/git/ignore` (XDG standard)
/// 3. `~/.gitignore` (legacy)
///
/// Returns the first path that exists, or None if none exist.
fn get_global_excludes_path() -> Option<PathBuf> {
    // Try git config core.excludesfile first
    if let Ok(output) = Command::new("git")
        .args(["config", "--global", "core.excludesfile"])
        .output()
    {
        if output.status.success() {
            let config_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !config_path.is_empty() {
                // Expand ~ if present
                let expanded = if let Some(stripped) = config_path.strip_prefix("~/") {
                    if let Some(home) = dirs::home_dir() {
                        home.join(stripped)
                    } else {
                        PathBuf::from(&config_path)
                    }
                } else {
                    PathBuf::from(&config_path)
                };
                if expanded.exists() {
                    return Some(expanded);
                }
            }
        }
    }

    // Try XDG standard location
    if let Some(home) = dirs::home_dir() {
        let xdg_path = home.join(".config/git/ignore");
        if xdg_path.exists() {
            return Some(xdg_path);
        }

        // Try legacy location
        let legacy_path = home.join(".gitignore");
        if legacy_path.exists() {
            return Some(legacy_path);
        }
    }

    None
}

/// Collect all `.gitignore` files from repository root to a target directory.
///
/// Walks from `repo_root` toward `target_dir`, collecting all `.gitignore` files
/// along the path. Files are returned in order from root to deepest (proper precedence).
///
/// # Arguments
///
/// * `repo_root` - The root directory of the git repository
/// * `target_dir` - The directory containing the file being checked
///
/// # Returns
///
/// A vector of paths to `.gitignore` files, ordered from root to target directory.
fn collect_gitignore_files(repo_root: &Path, target_dir: &Path) -> Vec<PathBuf> {
    let mut gitignore_files = Vec::new();

    // Get the relative path from repo_root to target_dir
    let rel_path = match target_dir.strip_prefix(repo_root) {
        Ok(p) => p,
        Err(_) => return gitignore_files, // target_dir not under repo_root
    };

    // Start from repo_root and walk toward target_dir
    let mut current = repo_root.to_path_buf();
    let gitignore_path = current.join(".gitignore");
    if gitignore_path.exists() {
        gitignore_files.push(gitignore_path);
    }

    // Walk through each component of the relative path
    for component in rel_path.components() {
        current.push(component);
        let gitignore_path = current.join(".gitignore");
        if gitignore_path.exists() {
            gitignore_files.push(gitignore_path);
        }
    }

    gitignore_files
}

/// Find the git repository root by walking up from a starting path.
///
/// Looks for a `.git` directory in the starting path and each parent directory.
/// Returns the directory containing `.git`, or None if not found.
///
/// # Arguments
///
/// * `start_path` - The path to start searching from
///
/// # Returns
///
/// Returns `Some(PathBuf)` with the repository root, or `None` if not in a git repository.
#[must_use]
pub fn find_git_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent()?.to_path_buf()
    } else {
        start_path.to_path_buf()
    };

    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None,
        }
    }
}

/// A checker for git-ignore patterns.
///
/// Wraps the `ignore::gitignore::Gitignore` to provide a simple API for checking
/// if paths match gitignore patterns. Loads patterns from global excludes,
/// repository excludes, and all nested `.gitignore` files.
pub struct GitIgnoreChecker {
    gitignore: Gitignore,
    repo_root: PathBuf,
}

impl GitIgnoreChecker {
    /// Creates a new `GitIgnoreChecker` by loading gitignore patterns from multiple sources.
    ///
    /// Loads patterns in proper precedence order (lowest to highest priority):
    /// 1. Global git excludes (`~/.config/git/ignore`, `core.excludesfile`, etc.)
    /// 2. Repository-specific excludes (`.git/info/exclude`)
    /// 3. Root `.gitignore`
    /// 4. Nested `.gitignore` files from root to target directory
    ///
    /// # Arguments
    ///
    /// * `repo_root` - The root directory of the git repository
    ///
    /// # Returns
    ///
    /// Returns a `GitIgnoreChecker` instance. If files cannot be parsed, logs warnings
    /// but continues (treating unparseable patterns as non-matching).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use conclaude::gitignore::GitIgnoreChecker;
    /// use std::path::Path;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let checker = GitIgnoreChecker::new(Path::new("/path/to/repo"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(repo_root: &Path) -> Result<Self> {
        let mut builder = GitignoreBuilder::new(repo_root);

        // 1. Load global git excludes (lowest priority)
        if let Some(global_path) = get_global_excludes_path() {
            if let Some(e) = builder.add(&global_path) {
                eprintln!(
                    "Warning: Failed to parse global git excludes at {}: {}",
                    global_path.display(),
                    e
                );
            }
        }

        // 2. Load repository-specific excludes (.git/info/exclude)
        let repo_exclude_path = repo_root.join(".git/info/exclude");
        if repo_exclude_path.exists() {
            if let Some(e) = builder.add(&repo_exclude_path) {
                eprintln!(
                    "Warning: Failed to parse repository excludes at {}: {}",
                    repo_exclude_path.display(),
                    e
                );
            }
        }

        // 3. Load root .gitignore
        let root_gitignore = repo_root.join(".gitignore");
        if root_gitignore.exists() {
            if let Some(e) = builder.add(&root_gitignore) {
                eprintln!(
                    "Warning: Failed to parse .gitignore at {}: {}",
                    root_gitignore.display(),
                    e
                );
            }
        }

        // Note: Nested .gitignore files will be loaded on-demand in is_ignored()
        // because they depend on the target file's location

        let gitignore = builder
            .build()
            .context("Failed to build gitignore matcher")?;

        Ok(Self {
            gitignore,
            repo_root: repo_root.to_path_buf(),
        })
    }

    /// Checks if a path matches gitignore patterns.
    ///
    /// Dynamically loads nested `.gitignore` files for the specific path being checked,
    /// ensuring proper precedence of patterns.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to check (should be relative to repository root, but absolute paths work too)
    ///
    /// # Returns
    ///
    /// Returns `(bool, Option<String>)` - whether the path is ignored and the matching pattern if found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use conclaude::gitignore::GitIgnoreChecker;
    /// use std::path::Path;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let checker = GitIgnoreChecker::new(Path::new("/path/to/repo"))?;
    /// let (is_ignored, pattern) = checker.is_ignored(Path::new("node_modules/foo.js"));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_ignored(&self, path: &Path) -> (bool, Option<String>) {
        // Convert to relative path if it's absolute
        let relative_path = if path.is_absolute() {
            match path.strip_prefix(&self.repo_root) {
                Ok(rel) => rel,
                Err(_) => path, // If not under repo root, use as-is
            }
        } else {
            path
        };

        // Get the directory containing the target file
        let target_dir = if relative_path.parent().is_some() {
            self.repo_root.join(relative_path.parent().unwrap())
        } else {
            self.repo_root.clone()
        };

        // Collect nested .gitignore files for this specific path
        let nested_gitignores = collect_gitignore_files(&self.repo_root, &target_dir);

        // If we have nested gitignores that aren't already in the base matcher,
        // build a new matcher; otherwise use the base matcher we already have.
        let root_gitignore_path = self.repo_root.join(".gitignore");
        let needs_nested_matcher = !(nested_gitignores.is_empty()
            || nested_gitignores.len() == 1 && nested_gitignores[0] == root_gitignore_path);

        let matcher = if needs_nested_matcher {
            // Build a new matcher with all gitignore files including nested ones
            let mut builder = GitignoreBuilder::new(&self.repo_root);

            // Re-add global excludes
            if let Some(global_path) = get_global_excludes_path() {
                let _ = builder.add(&global_path);
            }

            // Re-add repository excludes
            let repo_exclude_path = self.repo_root.join(".git/info/exclude");
            if repo_exclude_path.exists() {
                let _ = builder.add(&repo_exclude_path);
            }

            // Add all gitignore files in order (root to deepest)
            for gitignore_path in &nested_gitignores {
                if let Some(e) = builder.add(gitignore_path) {
                    eprintln!(
                        "Warning: Failed to parse .gitignore at {}: {}",
                        gitignore_path.display(),
                        e
                    );
                }
            }

            match builder.build() {
                Ok(gi) => gi,
                Err(_) => return (false, None), // On error, treat as non-ignored
            }
        } else {
            // Use the base matcher if no nested gitignores
            self.gitignore.clone()
        };

        // Check if the path matches gitignore patterns
        // We need to check both as a file and as a directory, because:
        // 1. node_modules/ only matches directories
        // 2. *.log matches files
        // We check the parent components as directories to handle nested paths like node_modules/foo.js

        // First check if this exact path is ignored as a file
        let file_match = matcher.matched(relative_path, false);
        if file_match.is_ignore() {
            let pattern = file_match.inner().map(|g| g.original().to_string());
            return (true, pattern);
        }

        // Then check each parent directory component
        // This handles cases like "node_modules/" ignoring "node_modules/foo.js"
        for ancestor in relative_path.ancestors().skip(1) {
            let dir_match = matcher.matched(ancestor, true);
            if dir_match.is_ignore() {
                let pattern = dir_match.inner().map(|g| g.original().to_string());
                return (true, pattern);
            }
        }

        (false, None)
    }
}

/// Convenience function to check if a path is git-ignored.
///
/// This is a simplified API that creates a `GitIgnoreChecker` and checks a single path.
/// Use this for one-off checks. For multiple checks, create a `GitIgnoreChecker` instance
/// and reuse it.
///
/// # Arguments
///
/// * `path` - The file path to check
/// * `repo_root` - The root directory of the git repository
///
/// # Returns
///
/// Returns a tuple of:
/// * `bool` - Whether the path is ignored
/// * `Option<String>` - The matching pattern (if ignored), useful for error messages
///
/// # Examples
///
/// ```rust
/// use conclaude::gitignore::is_path_git_ignored;
/// use std::path::Path;
///
/// # fn example() -> anyhow::Result<()> {
/// let (is_ignored, pattern) = is_path_git_ignored(
///     Path::new("node_modules/foo.js"),
///     Path::new("/path/to/repo")
/// )?;
///
/// if is_ignored {
///     println!("File is ignored by pattern: {:?}", pattern);
/// }
/// # Ok(())
/// # }
/// ```
pub fn is_path_git_ignored(path: &Path, repo_root: &Path) -> Result<(bool, Option<String>)> {
    let checker = GitIgnoreChecker::new(repo_root)?;
    let (is_ignored, pattern) = checker.is_ignored(path);
    Ok((is_ignored, pattern))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test repository with a .gitignore file
    fn create_test_repo(gitignore_content: &str) -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");
        fs::write(&gitignore_path, gitignore_content)?;
        Ok(temp_dir)
    }

    #[test]
    fn test_basic_gitignore_patterns() -> Result<()> {
        let temp_dir = create_test_repo("node_modules/\n*.log\ntarget/\n")?;
        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // Should be ignored (with pattern)
        let (is_ignored, pattern) = checker.is_ignored(Path::new("node_modules/foo.js"));
        assert!(is_ignored);
        assert_eq!(pattern, Some("node_modules/".to_string()));

        let (is_ignored, pattern) = checker.is_ignored(Path::new("debug.log"));
        assert!(is_ignored);
        assert_eq!(pattern, Some("*.log".to_string()));

        let (is_ignored, _) = checker.is_ignored(Path::new("target/release/app"));
        assert!(is_ignored);

        // Should not be ignored
        let (is_ignored, pattern) = checker.is_ignored(Path::new("src/main.rs"));
        assert!(!is_ignored);
        assert!(pattern.is_none());

        let (is_ignored, _) = checker.is_ignored(Path::new("README.md"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_no_gitignore_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // With no .gitignore, nothing should be ignored
        let (is_ignored, _) = checker.is_ignored(Path::new("node_modules/foo.js"));
        assert!(!is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("any/file.txt"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_negation_patterns() -> Result<()> {
        let temp_dir = create_test_repo("*.log\n!important.log\n")?;
        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // Regular .log files should be ignored
        let (is_ignored, _) = checker.is_ignored(Path::new("debug.log"));
        assert!(is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("error.log"));
        assert!(is_ignored);

        // important.log should NOT be ignored (negation pattern)
        let (is_ignored, _) = checker.is_ignored(Path::new("important.log"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_subdirectory_patterns() -> Result<()> {
        let temp_dir = create_test_repo("build/\n")?;
        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        let (is_ignored, _) = checker.is_ignored(Path::new("build/output.js"));
        assert!(is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("build/nested/file.txt"));
        assert!(is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("src/build.rs"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_corrupt_gitignore_handling() -> Result<()> {
        // Create a temp directory with an invalid/corrupt .gitignore
        // The ignore crate is fairly permissive, so we'll just test that
        // the checker handles it gracefully
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");

        // Write some edge case content (most patterns will parse fine though)
        fs::write(&gitignore_path, "node_modules/\n\n# comment\n*.log\n")?;

        // Should not panic or error
        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // Should still work with valid patterns
        let (is_ignored, _) = checker.is_ignored(Path::new("test.log"));
        assert!(is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("test.rs"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_find_git_root() -> Result<()> {
        // Test that find_git_root correctly locates .git directory
        let temp_dir = TempDir::new()?;
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir)?;

        // From repo root
        assert_eq!(
            find_git_root(temp_dir.path()),
            Some(temp_dir.path().to_path_buf())
        );

        // From subdirectory
        let sub_dir = temp_dir.path().join("src/nested");
        fs::create_dir_all(&sub_dir)?;
        assert_eq!(find_git_root(&sub_dir), Some(temp_dir.path().to_path_buf()));

        // Non-git directory
        let non_git = TempDir::new()?;
        assert_eq!(find_git_root(non_git.path()), None);

        Ok(())
    }

    #[test]
    fn test_nested_gitignore_files() -> Result<()> {
        // Create a repo with nested .gitignore files
        let temp_dir = TempDir::new()?;

        // Root .gitignore ignores *.log
        fs::write(temp_dir.path().join(".gitignore"), "*.log\n")?;

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("src/nested");
        fs::create_dir_all(&nested_dir)?;

        // Nested .gitignore ignores *.tmp but un-ignores important.log
        fs::write(nested_dir.join(".gitignore"), "*.tmp\n!important.log\n")?;

        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // Root level: .log files should be ignored
        let (is_ignored, _) = checker.is_ignored(Path::new("debug.log"));
        assert!(is_ignored);

        // Nested level: .log files should still be ignored by root rule
        let (is_ignored, _) = checker.is_ignored(Path::new("src/nested/error.log"));
        assert!(is_ignored);

        // Nested level: .tmp files should be ignored by nested rule
        let (is_ignored, _) = checker.is_ignored(Path::new("src/nested/cache.tmp"));
        assert!(is_ignored);

        // Root level: .tmp files should NOT be ignored (nested rule doesn't apply)
        let (is_ignored, _) = checker.is_ignored(Path::new("cache.tmp"));
        assert!(!is_ignored);

        // Nested level: important.log should NOT be ignored (negation in nested .gitignore)
        let (is_ignored, _) = checker.is_ignored(Path::new("src/nested/important.log"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_repository_exclude_file() -> Result<()> {
        // Create a repo with .git/info/exclude
        let temp_dir = TempDir::new()?;
        let git_info_dir = temp_dir.path().join(".git/info");
        fs::create_dir_all(&git_info_dir)?;

        // .git/info/exclude ignores *.secret
        fs::write(git_info_dir.join("exclude"), "*.secret\n")?;

        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // Files matching .git/info/exclude should be ignored
        let (is_ignored, _) = checker.is_ignored(Path::new("passwords.secret"));
        assert!(is_ignored);

        let (is_ignored, _) = checker.is_ignored(Path::new("src/api.secret"));
        assert!(is_ignored);

        // Other files should not be ignored
        let (is_ignored, _) = checker.is_ignored(Path::new("readme.txt"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_collect_gitignore_files() -> Result<()> {
        // Test the helper function that collects nested .gitignore files
        let temp_dir = TempDir::new()?;

        // Create nested structure with .gitignore files
        fs::write(temp_dir.path().join(".gitignore"), "root\n")?;

        let level1 = temp_dir.path().join("level1");
        fs::create_dir(&level1)?;
        fs::write(level1.join(".gitignore"), "level1\n")?;

        let level2 = level1.join("level2");
        fs::create_dir(&level2)?;
        fs::write(level2.join(".gitignore"), "level2\n")?;

        // Collect gitignores for level2
        let gitignores = collect_gitignore_files(temp_dir.path(), &level2);

        assert_eq!(gitignores.len(), 3);
        assert!(gitignores[0].ends_with(".gitignore"));
        assert!(gitignores[1].to_str().unwrap().contains("level1"));
        assert!(gitignores[2].to_str().unwrap().contains("level2"));

        // Collect for level1 (should only get root and level1)
        let gitignores = collect_gitignore_files(temp_dir.path(), &level1);
        assert_eq!(gitignores.len(), 2);

        // Collect for root (should only get root)
        let gitignores = collect_gitignore_files(temp_dir.path(), temp_dir.path());
        assert_eq!(gitignores.len(), 1);

        Ok(())
    }

    #[test]
    fn test_deeply_nested_gitignore() -> Result<()> {
        // Test that deeply nested .gitignore files work correctly
        let temp_dir = TempDir::new()?;

        // Root: ignore *.log
        fs::write(temp_dir.path().join(".gitignore"), "*.log\n")?;

        // Create deep nesting: a/b/c/d/
        let deep_path = temp_dir.path().join("a/b/c/d");
        fs::create_dir_all(&deep_path)?;

        // Middle level (a/b/): ignore *.tmp
        let mid_path = temp_dir.path().join("a/b");
        fs::write(mid_path.join(".gitignore"), "*.tmp\n")?;

        // Deep level (a/b/c/d/): ignore *.cache
        fs::write(deep_path.join(".gitignore"), "*.cache\n")?;

        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // All levels should see root .log rule
        let (is_ignored, _) = checker.is_ignored(Path::new("test.log"));
        assert!(is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("a/b/c/d/deep.log"));
        assert!(is_ignored);

        // .tmp should be ignored from a/b/ down
        let (is_ignored, _) = checker.is_ignored(Path::new("a/b/test.tmp"));
        assert!(is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("a/b/c/d/deep.tmp"));
        assert!(is_ignored);

        // .tmp should NOT be ignored at root or a/
        let (is_ignored, _) = checker.is_ignored(Path::new("test.tmp"));
        assert!(!is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("a/test.tmp"));
        assert!(!is_ignored);

        // .cache should only be ignored in a/b/c/d/
        let (is_ignored, _) = checker.is_ignored(Path::new("a/b/c/d/data.cache"));
        assert!(is_ignored);
        let (is_ignored, _) = checker.is_ignored(Path::new("a/b/data.cache"));
        assert!(!is_ignored);

        Ok(())
    }

    #[test]
    fn test_nested_gitignore_without_root_file() -> Result<()> {
        // Repository has no root .gitignore, only a nested one
        let temp_dir = TempDir::new()?;

        let nested_dir = temp_dir.path().join("src");
        fs::create_dir_all(&nested_dir)?;
        fs::write(nested_dir.join(".gitignore"), "*.secret\n")?;

        let checker = GitIgnoreChecker::new(temp_dir.path())?;

        // Files under the nested directory should respect its .gitignore
        let (is_ignored, _) = checker.is_ignored(Path::new("src/password.secret"));
        assert!(is_ignored);

        // Files outside the nested directory should not be affected
        let (is_ignored, _) = checker.is_ignored(Path::new("password.secret"));
        assert!(!is_ignored);

        Ok(())
    }
}

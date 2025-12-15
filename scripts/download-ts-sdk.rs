use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;

// Constants
const TARGET_DIR: &str = ".claude/contexts/claude-agent-sdk-ts";
const PACKAGE_NAME: &str = "@anthropic-ai/claude-agent-sdk";
const REGISTRY_URL: &str = "https://registry.npmjs.org/@anthropic-ai/claude-agent-sdk";

// ANSI color codes matching original script
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const RED: &str = "\x1b[0;31m";
const RESET: &str = "\x1b[0m";

// Helper functions for colored output
fn info(msg: &str) {
    println!("{}[INFO]{} {}", GREEN, RESET, msg);
}

fn warn(msg: &str) {
    println!("{}[WARN]{} {}", YELLOW, RESET, msg);
}

fn error(msg: &str) {
    eprintln!("{}[ERROR]{} {}", RED, RESET, msg);
}

// Fetch package metadata from npm registry
async fn fetch_package_metadata() -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("download-ts-sdk/1.0")
        .build()?;

    let response = client
        .get(REGISTRY_URL)
        .send()
        .await
        .context("Failed to fetch package metadata from npm registry")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch package metadata: HTTP {}",
            response.status()
        );
    }

    let json: Value = response
        .json()
        .await
        .context("Failed to parse package metadata JSON")?;

    // Extract latest version's tarball URL
    let tarball_url = json
        .get("dist-tags")
        .and_then(|dt| dt.get("latest"))
        .and_then(|v| v.as_str())
        .and_then(|version| {
            json.get("versions")
                .and_then(|versions| versions.get(version))
                .and_then(|ver_obj| ver_obj.get("dist"))
                .and_then(|dist| dist.get("tarball"))
                .and_then(|url| url.as_str())
        })
        .context("Failed to extract tarball URL from package metadata")?;

    Ok(tarball_url.to_string())
}

// Download tarball from URL
async fn download_tarball(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .user_agent("download-ts-sdk/1.0")
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to download tarball")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download tarball: HTTP {}", response.status());
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read tarball bytes")?;

    Ok(bytes.to_vec())
}

// Extract tarball to target directory, stripping 'package/' prefix
fn extract_tarball(tarball_bytes: &[u8], target_dir: &Path) -> Result<()> {
    let decoder = GzDecoder::new(tarball_bytes);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().context("Failed to read tarball entries")? {
        let mut entry = entry.context("Failed to read tarball entry")?;

        // Get the path from the entry
        let path = entry.path().context("Failed to get entry path")?;
        let path_str = path.to_string_lossy();

        // Strip 'package/' prefix (npm tarballs contain files under 'package/' directory)
        let stripped_path = if let Some(stripped) = path_str.strip_prefix("package/") {
            PathBuf::from(stripped)
        } else if path_str == "package" {
            // Skip the 'package' directory itself
            continue;
        } else {
            // Keep the path as-is if it doesn't have the prefix
            path.to_path_buf()
        };

        // Build the full target path
        let target_path = target_dir.join(&stripped_path);

        // Create parent directories if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // Extract the file
        entry
            .unpack(&target_path)
            .with_context(|| format!("Failed to extract file: {:?}", target_path))?;
    }

    Ok(())
}

// Remove existing installation if it exists
fn remove_existing_installation() -> Result<bool> {
    let target_path = Path::new(TARGET_DIR);
    let package_json = target_path.join("package.json");

    if package_json.exists() {
        info("Removing old installation...");
        fs::remove_dir_all(target_path)
            .with_context(|| format!("Failed to remove existing installation at {}", TARGET_DIR))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// Create target directory
fn create_target_directory() -> Result<()> {
    fs::create_dir_all(TARGET_DIR)
        .with_context(|| format!("Failed to create directory: {}", TARGET_DIR))?;
    Ok(())
}

// Read installed version from package.json
fn read_installed_version() -> Result<String> {
    let package_json_path = Path::new(TARGET_DIR).join("package.json");
    let content = fs::read_to_string(&package_json_path)
        .context("Failed to read package.json")?;

    let json: Value = serde_json::from_str(&content)
        .context("Failed to parse package.json")?;

    let version = json
        .get("version")
        .and_then(|v| v.as_str())
        .context("Failed to extract version from package.json")?;

    Ok(version.to_string())
}

// Check if src directory exists
fn check_src_directory() -> bool {
    Path::new(TARGET_DIR).join("src").exists()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Print header
    println!();
    info("TypeScript SDK Download Script");
    println!();

    // Fetch package metadata
    info(&format!("Downloading {}...", PACKAGE_NAME));
    let tarball_url = fetch_package_metadata().await.map_err(|e| {
        error("Failed to download package metadata");
        error("This could be due to:");
        error("  - Network connectivity issues");
        error("  - npm registry unavailability");
        error("Try running the script again or check your internet connection");
        e
    })?;

    // Download tarball
    let tarball_bytes = download_tarball(&tarball_url).await.map_err(|e| {
        error("Failed to download package tarball");
        error("This could be due to:");
        error("  - Network connectivity issues");
        error("  - npm registry unavailability");
        error("Try running the script again or check your internet connection");
        e
    })?;

    // Extract tarball
    info("Extracting TypeScript SDK source files...");

    // Remove old installation if exists
    remove_existing_installation()?;

    // Create target directory
    create_target_directory()?;

    // Install
    info(&format!("Installing to {}...", TARGET_DIR));
    extract_tarball(&tarball_bytes, Path::new(TARGET_DIR)).map_err(|e| {
        error("Failed to extract package tarball");
        e
    })?;

    info("TypeScript SDK successfully installed!");

    // Show success message
    println!();
    info(&format!(
        "✓ TypeScript SDK source files are available at: {}",
        TARGET_DIR
    ));

    if check_src_directory() {
        info(&format!("✓ Source code location: {}/src", TARGET_DIR));
    }

    match read_installed_version() {
        Ok(version) => {
            info(&format!("✓ Version: {}", version));
        }
        Err(_) => {
            warn("Could not determine installed version");
        }
    }

    println!();

    Ok(())
}

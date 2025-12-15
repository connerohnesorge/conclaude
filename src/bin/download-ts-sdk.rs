use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::process;

use flate2::read::GzDecoder;
use reqwest;
use serde_json::Value;
use tar::Archive;

// Constants
const PACKAGE_NAME: &str = "@anthropic-ai/claude-agent-sdk";
const REGISTRY_URL: &str = "https://registry.npmjs.org/@anthropic-ai/claude-agent-sdk";
const TARGET_DIR: &str = ".claude/contexts/claude-agent-sdk-ts";

// ANSI color codes
const GREEN: &str = "\x1b[0;32m";
const RED: &str = "\x1b[0;31m";
const RESET: &str = "\x1b[0m";

// Color helper functions
fn info(message: &str) {
    println!("{}[INFO]{} {}", GREEN, RESET, message);
}

fn error(message: &str) {
    eprintln!("{}[ERROR]{} {}", RED, RESET, message);
}

// Fetch package metadata from npm registry
async fn fetch_package_metadata() -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(REGISTRY_URL).await?;
    let json: Value = response.json().await?;

    // Extract the latest version's tarball URL
    let latest_version = json["dist-tags"]["latest"]
        .as_str()
        .ok_or("Failed to find latest version")?;

    let tarball_url = json["versions"][latest_version]["dist"]["tarball"]
        .as_str()
        .ok_or("Failed to find tarball URL")?;

    Ok(tarball_url.to_string())
}

// Download tarball from URL
async fn download_tarball(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

// Extract tarball to target directory, stripping the "package/" prefix
fn extract_tarball(tarball_bytes: &[u8], target_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cursor = Cursor::new(tarball_bytes);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Strip "package/" prefix from paths
        let stripped_path = path.strip_prefix("package").unwrap_or(&path);

        // Skip if stripped path is empty (i.e., it was just "package/")
        if stripped_path.as_os_str().is_empty() {
            continue;
        }

        let dest_path = Path::new(target_dir).join(stripped_path);

        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Extract the file
        entry.unpack(&dest_path)?;
    }

    Ok(())
}

// Remove existing installation if present
fn remove_existing_installation(target_dir: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let package_json_path = Path::new(target_dir).join("package.json");

    if package_json_path.exists() {
        info("Removing old installation...");
        fs::remove_dir_all(target_dir)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// Create target directory
fn create_target_directory(target_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(target_dir)?;
    Ok(())
}

// Read installed version from package.json
fn read_installed_version(target_dir: &str) -> Option<String> {
    let package_json_path = Path::new(target_dir).join("package.json");

    if let Ok(content) = fs::read_to_string(package_json_path) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            return json["version"].as_str().map(|s| s.to_string());
        }
    }

    None
}

// Check if src directory exists
fn check_src_directory(target_dir: &str) -> bool {
    Path::new(target_dir).join("src").exists()
}

#[tokio::main]
async fn main() {
    println!();
    info("TypeScript SDK Download Script");
    println!();

    // Download package metadata
    info(&format!("Downloading {}...", PACKAGE_NAME));
    let tarball_url = match fetch_package_metadata().await {
        Ok(url) => url,
        Err(e) => {
            error(&format!("Failed to fetch package metadata: {}", e));
            error("This could be due to:");
            error("  - Network connectivity issues");
            error("  - npm registry unavailability");
            error("Try running the script again or check your internet connection");
            process::exit(1);
        }
    };

    // Download tarball
    let tarball_bytes = match download_tarball(&tarball_url).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error(&format!("Failed to download tarball: {}", e));
            process::exit(1);
        }
    };

    info("Extracting TypeScript SDK source files...");

    // Remove old installation if exists
    if let Err(e) = remove_existing_installation(TARGET_DIR) {
        error(&format!("Failed to remove old installation: {}", e));
        process::exit(1);
    }

    // Create target directory
    info(&format!("Installing to {}...", TARGET_DIR));
    if let Err(e) = create_target_directory(TARGET_DIR) {
        error(&format!("Failed to create target directory: {}", e));
        error("Check that you have write permissions in this location");
        process::exit(1);
    }

    // Extract tarball
    if let Err(e) = extract_tarball(&tarball_bytes, TARGET_DIR) {
        error(&format!("Failed to extract tarball: {}", e));
        process::exit(1);
    }

    info("TypeScript SDK successfully installed!");
    println!();

    // Display success information
    info(&format!("✓ TypeScript SDK source files are available at: {}", TARGET_DIR));

    if check_src_directory(TARGET_DIR) {
        info(&format!("✓ Source code location: {}/src", TARGET_DIR));
    }

    if let Some(version) = read_installed_version(TARGET_DIR) {
        info(&format!("✓ Version: {}", version));
    }

    println!();
}

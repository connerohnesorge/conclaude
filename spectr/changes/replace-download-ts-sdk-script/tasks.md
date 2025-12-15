## 1. Dependencies & Setup

- [ ] 1.1 Add `reqwest` dependency with `rustls-tls` feature to Cargo.toml
- [ ] 1.2 Add `flate2` dependency to Cargo.toml
- [ ] 1.3 Add `tar` dependency to Cargo.toml
- [ ] 1.4 Add `[[bin]]` entry for `download-ts-sdk` pointing to `scripts/download-ts-sdk.rs`

## 2. Core Implementation

- [ ] 2.1 Create `scripts/download-ts-sdk.rs` with tokio main entry point
- [ ] 2.2 Implement ANSI color helper functions (info, warn, error) matching original script format:
  - Green `\x1b[0;32m` for `[INFO]`
  - Yellow `\x1b[1;33m` for `[WARN]`
  - Red `\x1b[0;31m` for `[ERROR]` (to stderr)
  - Reset `\x1b[0m`
- [ ] 2.3 Define constants:
  - `TARGET_DIR = ".claude/contexts/claude-agent-sdk-ts"`
  - `PACKAGE_NAME = "@anthropic-ai/claude-agent-sdk"`
  - `REGISTRY_URL = "https://registry.npmjs.org/@anthropic-ai/claude-agent-sdk"`
- [ ] 2.4 Implement `fetch_package_metadata()`: GET registry URL, parse JSON, extract `dist.tarball` URL from `dist-tags.latest` version
- [ ] 2.5 Implement `download_tarball()`: GET tarball URL, return bytes
- [ ] 2.6 Implement `extract_tarball()`:
  - Use flate2::read::GzDecoder for gzip decompression
  - Use tar::Archive for extraction
  - Strip `package/` prefix from all paths (npm tarball structure)
  - Write files to TARGET_DIR
- [ ] 2.7 Implement `remove_existing_installation()`: check if TARGET_DIR/package.json exists, if so remove TARGET_DIR
- [ ] 2.8 Implement `create_target_directory()`: create TARGET_DIR with mkdir_all
- [ ] 2.9 Implement `read_installed_version()`: parse TARGET_DIR/package.json, extract "version" field
- [ ] 2.10 Implement `check_src_directory()`: check if TARGET_DIR/src exists

## 3. Main Flow

- [ ] 3.1 Implement main() with exact message sequence matching original script:
  1. Print header: `[INFO] TypeScript SDK Download Script`
  2. `[INFO] Downloading @anthropic-ai/claude-agent-sdk...`
  3. Fetch metadata and download tarball
  4. `[INFO] Extracting TypeScript SDK source files...`
  5. Check and remove existing installation (if exists, print `[INFO] Removing old installation...`)
  6. Create target directory
  7. Extract tarball
  8. `[INFO] Installing to .claude/contexts/claude-agent-sdk-ts...`
  9. `[INFO] TypeScript SDK successfully installed!`
  10. Print blank line
  11. `[INFO] ✓ TypeScript SDK source files are available at: .claude/contexts/claude-agent-sdk-ts`
  12. If src/ exists: `[INFO] ✓ Source code location: .claude/contexts/claude-agent-sdk-ts/src`
  13. `[INFO] ✓ Version: {version}`
  14. Print blank line
- [ ] 3.2 Implement error handling with descriptive messages matching original script style

## 4. Validation

- [ ] 4.1 Verify binary compiles: `cargo build --bin download-ts-sdk`
- [ ] 4.2 Run binary and confirm SDK downloads to correct location
- [ ] 4.3 Verify extracted file structure contains package.json, src/, etc.
- [ ] 4.4 Verify version is correctly extracted and displayed
- [ ] 4.5 Test re-running binary replaces existing installation cleanly
- [ ] 4.6 Test error messages appear correctly for network failures

## 5. Cleanup

- [ ] 5.1 Remove `scripts/download-ts-sdk.sh`
- [ ] 5.2 Search codebase for references to the shell script and update if needed

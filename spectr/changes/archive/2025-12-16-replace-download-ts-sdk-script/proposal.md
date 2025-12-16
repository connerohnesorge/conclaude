# Change: Replace download-ts-sdk.sh with Rust binary

## Why
The shell script `scripts/download-ts-sdk.sh` requires npm to be installed just to download and extract an npm package. A Rust binary can fetch the package tarball directly from the npm registry API, eliminating the npm dependency and providing consistent cross-platform behavior.

## What Changes
- Remove `scripts/download-ts-sdk.sh` shell script
- Add `scripts/download-ts-sdk.rs` as a new Cargo binary
- Add `[[bin]]` entry in `Cargo.toml` for `download-ts-sdk`
- Directly fetch package metadata and tarball from npm registry (registry.npmjs.org)
- Extract tarball using Rust-native tar/gzip decompression
- Provide consistent colored output and error handling across all platforms

## Design Decisions

### Versioning
**Decision**: Always fetch latest version (no version pinning flag)
**Rationale**: Matches original script behavior. Keeps implementation simple.

### Target Path
**Decision**: Hardcoded to `.claude/contexts/claude-agent-sdk-ts`
**Rationale**: Matches original script. Single-purpose tool doesn't need configurability.

### HTTP Client
**Decision**: `reqwest` with tokio async runtime
**Rationale**: Project already uses tokio. reqwest is the standard async HTTP client.

### TLS Backend
**Decision**: Use `rustls-tls` feature (not native-tls)
**Rationale**: Pure Rust, no OpenSSL dependency, better cross-platform support.

### Colored Output
**Decision**: Manual ANSI escape codes
**Rationale**: No extra dependency. Matches original script's exact color codes.

### Temp Files
**Decision**: Stream directly to memory, no temp files needed
**Rationale**: The tarball is small (<5MB). Streaming avoids temp file cleanup complexity.

### Tarball Structure
**Decision**: Handle npm's `package/` prefix directory during extraction
**Rationale**: npm tarballs contain files under a `package/` directory. Must strip this prefix when extracting to target.

## Technical Details

### Package Information
- **Package name**: `@anthropic-ai/claude-agent-sdk`
- **Registry URL**: `https://registry.npmjs.org/@anthropic-ai/claude-agent-sdk`
- **Target directory**: `.claude/contexts/claude-agent-sdk-ts`

### ANSI Color Codes (matching original script)
- Green (INFO): `\x1b[0;32m`
- Yellow (WARN): `\x1b[1;33m`
- Red (ERROR): `\x1b[0;31m`
- Reset: `\x1b[0m`

### Message Format
- `[INFO] message` - informational (green)
- `[WARN] message` - warnings (yellow)
- `[ERROR] message` - errors to stderr (red)

### Success Output (matching original script)
```
[INFO] TypeScript SDK Download Script

[INFO] Downloading @anthropic-ai/claude-agent-sdk...
[INFO] Extracting TypeScript SDK source files...
[INFO] Removing old installation...  (if exists)
[INFO] Installing to .claude/contexts/claude-agent-sdk-ts...
[INFO] TypeScript SDK successfully installed!

[INFO] ✓ TypeScript SDK source files are available at: .claude/contexts/claude-agent-sdk-ts
[INFO] ✓ Source code location: .claude/contexts/claude-agent-sdk-ts/src  (if src/ exists)
[INFO] ✓ Version: x.y.z
```

## Impact
- **Affected specs**: tooling (new capability)
- **Affected code**:
  - `scripts/download-ts-sdk.sh` (removed)
  - `scripts/download-ts-sdk.rs` (new)
  - `Cargo.toml` (new binary entry, new dependencies)
- **New dependencies**:
  - `reqwest` (with `rustls-tls` feature) - HTTP client
  - `flate2` - gzip decompression
  - `tar` - tarball extraction

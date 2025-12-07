## 1. Replace Placeholder Content

- [x] 1.1 Update `docs/src/content/docs/index.mdx` with conclaude-branded homepage
  - Hero section with conclaude tagline and value proposition
  - Quick installation snippet
  - Feature cards highlighting key capabilities
  - Call-to-action buttons linking to guides and reference

- [x] 1.2 Replace `docs/src/content/docs/guides/example.md` with "Getting Started" guide
  - Rename to `getting-started.md`
  - Quick installation instructions
  - First configuration walkthrough
  - Running first hook demonstration
  - Link to full reference

- [x] 1.3 Replace `docs/src/content/docs/reference/example.md` with CLI Reference
  - Rename to `cli.md`
  - Document all CLI commands (init, validate, hook commands)
  - Exit codes and error handling
  - Environment variables reference

## 2. Add Essential Guides

- [x] 2.1 Create `docs/src/content/docs/guides/installation.md`
  - Shell script installation (recommended)
  - PowerShell installation (Windows)
  - npm package installation
  - Manual binary download
  - Nix flake installation
  - Building from source

- [x] 2.2 Create `docs/src/content/docs/guides/hooks.md`
  - Overview of the hook system
  - PreToolUse, PostToolUse, Stop hooks explained
  - Session lifecycle hooks
  - Subagent hooks
  - Example configurations for each hook type

## 3. Validation

- [x] 3.1 Verify all internal links work correctly
- [x] 3.2 Build documentation site and check for errors
- [x] 3.3 Verify navigation sidebar shows correct structure

# Change: Replace Placeholder Documentation with Conclaude Content

## Why

The documentation site currently contains generic Starlight template placeholder content that provides no value to users. The homepage welcomes visitors to "Starlight" and links to Starlight docs, the guides contain Diataxis framework explanations instead of actual guides, and the reference section has template text. This creates a poor user experience and fails to communicate conclaude's value proposition.

## What Changes

- **Homepage (`index.mdx`)**: Replace generic Starlight welcome with conclaude-branded hero section featuring installation, feature highlights, and quick links to guides
- **Guides (`guides/example.md`)**: Transform into a proper "Getting Started" guide covering installation, first configuration, and basic usage
- **Reference (`reference/example.md`)**: Convert to a CLI reference page documenting all available commands and options
- **New Guide (`guides/installation.md`)**: Add dedicated installation guide covering all installation methods
- **New Guide (`guides/hooks.md`)**: Add hooks overview guide explaining the hook system

## Impact

- Affected specs: `documentation`
- Affected code: `docs/src/content/docs/` (5 markdown files)
- User-facing change: Documentation will now be useful and specific to conclaude

# documentation Spec Delta

## ADDED Requirements

### Requirement: Contextual Menu Plugin Integration

The documentation site SHALL provide a contextual menu plugin that enables quick content actions directly from the user interface.

#### Scenario: Plugin is installed
- **WHEN** the documentation site dependencies are installed
- **THEN** the `starlight-contextual-menu` package exists in `docs/package.json` dependencies

#### Scenario: Plugin is configured in Astro config
- **WHEN** the Astro configuration is loaded
- **THEN** the `starlight-contextual-menu` plugin is imported and added to the Starlight plugins array

#### Scenario: Copy action is available
- **WHEN** a user right-clicks on a documentation page
- **THEN** a "Copy" action appears in the contextual menu to copy page content

#### Scenario: View markdown action is available
- **WHEN** a user right-clicks on a documentation page
- **THEN** a "View" action appears in the contextual menu to view raw markdown

#### Scenario: ChatGPT action is available
- **WHEN** a user right-clicks on a documentation page
- **THEN** a "ChatGPT" action appears in the contextual menu to open content in ChatGPT

#### Scenario: Claude action is available
- **WHEN** a user right-clicks on a documentation page
- **THEN** a "Claude" action appears in the contextual menu to open content in Claude

### Requirement: Contextual Menu Configuration

The system SHALL configure the contextual menu plugin with specific actions tailored for conclaude documentation users.

#### Scenario: Actions array is configured
- **WHEN** the plugin configuration is defined in `astro.config.mjs`
- **THEN** the `actions` option includes `["copy", "view", "chatgpt", "claude"]`

#### Scenario: Default plugin options are used
- **WHEN** optional configuration parameters are not specified
- **THEN** the plugin uses default values for `injectMarkdownRoutes` (true) and `hideMainActionLabel` (false)

## ADDED Requirements

### Requirement: Starlight Changelogs Plugin Integration

The documentation site SHALL integrate the starlight-changelogs plugin to display project release history from GitHub releases.

#### Scenario: Plugin is installed and configured

- **WHEN** the documentation site is built
- **THEN** the starlight-changelogs plugin is loaded and active in the Astro configuration

#### Scenario: Changelogs collection is available

- **WHEN** the site content is processed
- **THEN** a changelogs collection is defined in the content configuration using changelogsLoader

### Requirement: GitHub Provider Configuration

The documentation site SHALL use the GitHub provider to fetch and display releases from the conclaude repository.

#### Scenario: GitHub releases are fetched

- **WHEN** the documentation site builds or the changelog page is accessed
- **THEN** the plugin fetches release data from the conclaude GitHub repository

#### Scenario: Repository owner and name are configured

- **WHEN** the GitHub provider is initialized
- **THEN** it uses the correct repository owner and repository name for the conclaude project

#### Scenario: Changelog base path is configured

- **WHEN** a user navigates to changelog pages
- **THEN** changelog routes are available under the configured base path (e.g., /changelog/)

### Requirement: Documentation Site Branding

The documentation site SHALL be properly branded with the conclaude project name and repository link.

#### Scenario: Site title reflects project name

- **WHEN** the documentation site is loaded
- **THEN** the site title displays 'conclaude' instead of generic placeholder text

#### Scenario: GitHub social link points to conclaude repository

- **WHEN** a user clicks the GitHub social icon
- **THEN** they are directed to the conclaude GitHub repository

### Requirement: Changelog Navigation

The documentation site SHALL provide navigation access to the changelog section.

#### Scenario: Changelog appears in sidebar

- **WHEN** a user views the documentation site
- **THEN** a changelog navigation entry is visible in the sidebar

#### Scenario: Changelog link routes correctly

- **WHEN** a user clicks the changelog navigation entry
- **THEN** they are taken to the changelog index or overview page

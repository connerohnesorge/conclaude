## ADDED Requirements

### Requirement: Internal Link Validation

The documentation site SHALL validate all internal links during production builds using the starlight-links-validator plugin.

#### Scenario: Plugin is configured

- **WHEN** the Starlight configuration is loaded
- **THEN** the `starlightLinksValidator` plugin is included in the plugins array

#### Scenario: Internal links are validated on build

- **WHEN** a production build is executed (`npm run build`)
- **THEN** the plugin validates all internal links in Markdown and MDX files

#### Scenario: Broken internal link fails build

- **WHEN** a Markdown file contains a broken internal link (404)
- **THEN** the production build fails with an error indicating the broken link

#### Scenario: External links are ignored

- **WHEN** a Markdown file contains external links (http/https URLs)
- **THEN** the plugin ignores these links and does not validate them

#### Scenario: Hash links are validated

- **WHEN** a Markdown file links to a heading anchor (e.g., `#installation`)
- **THEN** the plugin validates that the target heading exists in the referenced page

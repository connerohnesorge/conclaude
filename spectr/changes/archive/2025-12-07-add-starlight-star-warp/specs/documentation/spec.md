## ADDED Requirements

### Requirement: Client-Side Search with Pagefind

The documentation site SHALL provide fast, client-side search functionality powered by the Pagefind search engine through the star-warp plugin.

#### Scenario: Plugin installed and configured

- **WHEN** the documentation site is built
- **THEN** the @inox-tools/star-warp plugin is installed as a dependency and integrated into the Astro configuration

#### Scenario: Search index generated at build time

- **WHEN** the site build process completes
- **THEN** Pagefind generates a search index of all documentation content

#### Scenario: Search interface available

- **WHEN** users access the documentation site
- **THEN** a search interface is available allowing users to query documentation content

#### Scenario: Search results returned instantly

- **WHEN** users enter a search query
- **THEN** results are returned from the client-side index without server requests

#### Scenario: Default configuration used

- **WHEN** the plugin is integrated
- **THEN** it uses minimal/default configuration settings without custom options

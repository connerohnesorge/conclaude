---
title: Changelog
description: All notable changes to conclaude
---

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.6] - 2025-12-18

### Features

- Add showCommand configuration option (#175) ([7e1ce37](https://github.com/connerohnesorge/conclaude/commit/7e1ce379018c76b16ab3d2649cf8a88403f59280))

### Documentation

- Add auto import of changelog from root of repo (#178) ([8d98e98](https://github.com/connerohnesorge/conclaude/commit/8d98e9893c122785d0d0bc39484660cef852333a))

### Miscellaneous Tasks

- **release:** Update CHANGELOG.md for v0.2.5 ([a8e3726](https://github.com/connerohnesorge/conclaude/commit/a8e3726f678d83976263ca4f2e6ca3a2ceeefb41))

### Spectr

- **archive:** Add-prompt-context-injection (#172) ([bc9bc28](https://github.com/connerohnesorge/conclaude/commit/bc9bc2800ff19db210309f24f4b9bc9153051144))
- **archive:** Replace-download-ts-sdk-script (#173) ([bdc6d90](https://github.com/connerohnesorge/conclaude/commit/bdc6d90db1212f2cba043a4de9a79e7b5a10c36a))
- **apply:** Add-stop-hook-pr-check-example (#174) ([dcce924](https://github.com/connerohnesorge/conclaude/commit/dcce92466fe0fc528386b697f425df14eae9ef1f))
- **archive:** Add-stop-hook-pr-check-example (#176) ([6f59475](https://github.com/connerohnesorge/conclaude/commit/6f594754f697e9f9ad28a7fd015dad92518d1463))
- **proposal:** Document-file-protection-example (#177) ([cbe1be8](https://github.com/connerohnesorge/conclaude/commit/cbe1be8dde798827f44ae2d11759dc81ab11d1c9))

## [0.2.5] - 2025-12-16

### Features

- Add CONCLAUDE_AGENT_NAME env var to SubagentStop hook (#157) ([5fb4116](https://github.com/connerohnesorge/conclaude/commit/5fb4116dfe83a8f0b6a6542641fece9be650ff4f))
- Replace download-ts-sdk.sh with Rust binary (#164) ([9f8c8f6](https://github.com/connerohnesorge/conclaude/commit/9f8c8f6d240178ad23be41ee1a28476f0bcfcd07))
- **init:** Add 10-minute timeout to generated Claude Code hooks (#168) ([1db7f62](https://github.com/connerohnesorge/conclaude/commit/1db7f6295a76fe7f3bfc2d6ac983029d470cf4d7))
- **hooks:** Add context injection for UserPromptSubmit hook (#170) ([3e4585a](https://github.com/connerohnesorge/conclaude/commit/3e4585a542cfbae4be2c15ec7cfc0e8fc07377a3))

### Bug Fixes

- Fix docs astro config (to use gh pages) ([4903a35](https://github.com/connerohnesorge/conclaude/commit/4903a3591c26bb6800d49f2617bad36049c7e5ee))
- Fix docs generation to use relative links and fix README ([91e3738](https://github.com/connerohnesorge/conclaude/commit/91e3738db04a44ec9a2558d773f666cb9a4bcd4e))
- Fix docs links ([383ed9b](https://github.com/connerohnesorge/conclaude/commit/383ed9b04aa9ba88c1281271ae3312ee55cf1a34))
- Fixed internal links in docs ([6a97f53](https://github.com/connerohnesorge/conclaude/commit/6a97f539d0790180bc7d9e6757f8769693c74e4f))
- Fix internal links of permission-request.md ([67e69da](https://github.com/connerohnesorge/conclaude/commit/67e69dac4527301e2636da8ed243b073d1518ba3))

### Refactoring

- Extract hooks tests to separate module for improved testability (#141) ([5092968](https://github.com/connerohnesorge/conclaude/commit/5092968a842cd420efd93f5bfb16170e7ca24a8c))
- Move conclaude-field-derive crate to crates/ directory (#143) ([abfef6d](https://github.com/connerohnesorge/conclaude/commit/abfef6d745d561bb470d35a118f486434c2aece6))
- [**breaking**] Reorganize hook commands under Hooks parent subcommand (#148) ([8f169e6](https://github.com/connerohnesorge/conclaude/commit/8f169e637fe7780432f385836bbb37d827db410a))
- Relocate generate-schema binary to src/bin/ (#161) ([92c2e65](https://github.com/connerohnesorge/conclaude/commit/92c2e657b86f854450269f5d47eafa18f1bad5f9))

### Miscellaneous Tasks

- **release:** Update CHANGELOG.md for v0.2.4 ([b8029d1](https://github.com/connerohnesorge/conclaude/commit/b8029d129b976cd44053131c23d57a8010b70013))

### Spectr

- **archive:** Add-starlight-links-validator (#138) ([23be06c](https://github.com/connerohnesorge/conclaude/commit/23be06c4735ff245f4f1e0f2ffcbbbc811943904))
- **archive:** Add-command-timeout (#135) ([b3a6376](https://github.com/connerohnesorge/conclaude/commit/b3a6376d837edeb93813c8c99c35789754e4340c))
- **archive:** Add-starlight-site-graph (#137) ([12292db](https://github.com/connerohnesorge/conclaude/commit/12292dbbf067ba9fa1d64fee3506bf43956f1745))
- **archive:** Add-config-documentation-generator (#136) ([2c1cfb8](https://github.com/connerohnesorge/conclaude/commit/2c1cfb8df3a634d6c892aab8b579362bf007fcca))
- **archive:** Add-contextual-menu-plugin (#142) ([ea65ebd](https://github.com/connerohnesorge/conclaude/commit/ea65ebdfd2d0aa6433e739d9382a2b24850a7539))
- **proposal:** Refactor-hooks-subcommand (#145) ([0714977](https://github.com/connerohnesorge/conclaude/commit/0714977a96db1d78e07d892453a38e4f413be081))
- **archive:** Refactor-hooks-tests-to-separate-file (#146) ([1961292](https://github.com/connerohnesorge/conclaude/commit/1961292e13ab7e1bd2f34ce3b323b6fe76b9a247))
- **archive:** Refactor-move-derive-crate-to-crates (#147) ([7ae4498](https://github.com/connerohnesorge/conclaude/commit/7ae4498ff6a2fd2cf4c2e0a91472c166b4f5c64e))
- Archive refactor-hooks-subcommand and fix cli-structure spec (#152) ([6fab96e](https://github.com/connerohnesorge/conclaude/commit/6fab96e76e10690fffe80da2720befe5f2090be8))
- **remove:** Extend-bash-command-validation (#153) ([8cf5aca](https://github.com/connerohnesorge/conclaude/commit/8cf5acab011b453d9762392f28e9a609b046670b))
- **proposal:** Replace-download-ts-sdk-script (#160) ([18f3d39](https://github.com/connerohnesorge/conclaude/commit/18f3d39fb89dfcd70605e1b8a4f5fee6c376c190))
- **proposal:** Relocate-generate-schema-binary (#159) ([589a672](https://github.com/connerohnesorge/conclaude/commit/589a672dc67268cd110a4ac129577b9dcfc4b7ef))
- **archive:** Relocate-generate-schema-binary (#163) ([717cb01](https://github.com/connerohnesorge/conclaude/commit/717cb0147da5b630d16732ea461d9471fe271f5c))
- **proposal:** Add-prompt-context-injection (#165) ([2ae712d](https://github.com/connerohnesorge/conclaude/commit/2ae712dad562770f4ec2629f95dcfb8ad85053d4))
- **proposal:** Add-init-hook-timeout (#166) ([a3082fc](https://github.com/connerohnesorge/conclaude/commit/a3082fc5e26082e702405e75d1a96e8561cc3e93))
- **proposal:** Add-show-command-config (#167) ([e8cbd03](https://github.com/connerohnesorge/conclaude/commit/e8cbd03b7888a55a40a6e75c992bf6bd72177baf))
- **archive:** Add-init-hook-timeout (#169) ([02473c9](https://github.com/connerohnesorge/conclaude/commit/02473c981f15ec6c5f2ab31a6843671acfb8b84d))
- **proposal:** Add-stop-hook-pr-check-example (#171) ([522d388](https://github.com/connerohnesorge/conclaude/commit/522d3881d91cd82d770ec7aafe8f35ad29b6c7cf))

## [0.2.4] - 2025-12-07

### Features

- [**breaking**] Remove rounds feature from stop hook configuration (#112) ([2109c5d](https://github.com/connerohnesorge/conclaude/commit/2109c5d05d336c4d7e3c9c73fe86700cbfba2555))
- Add optional timeout field for stop commands (#109) ([952ff50](https://github.com/connerohnesorge/conclaude/commit/952ff50f8ca43041d90a3da2eb5a58d1a07b9536))

### Documentation

- Replace placeholder content with conclaude documentation (#129) ([26774f5](https://github.com/connerohnesorge/conclaude/commit/26774f5d9136c3ddf5b3ceaee9fb295ddb2deebf))

### Miscellaneous Tasks

- **release:** Update CHANGELOG.md for v0.2.2 ([0cc14ab](https://github.com/connerohnesorge/conclaude/commit/0cc14ab253efaba248ba253de578af33804d59db))

### Spectr

- **proposal:** Fix-command-working-directory (#114) ([73ad865](https://github.com/connerohnesorge/conclaude/commit/73ad865339a445102f3c15e3f76523a050c34913))
- **apply:** Fix-command-working-directory (#115) ([031a97b](https://github.com/connerohnesorge/conclaude/commit/031a97b13e6b7af3ff84c570fc9274ee3adc4d37))
- **apply:** Add-git-block-example (#116) ([a98f069](https://github.com/connerohnesorge/conclaude/commit/a98f0692f9a59438f83b44157e25bd3e14a48ec3))
- **proposal:** Add-contextual-menu-plugin (#124) ([b82b405](https://github.com/connerohnesorge/conclaude/commit/b82b4054af1abe0d7fac5f38c848838cb3a18e20))
- **proposal:** Add-starlight-star-warp (#123) ([8f372a3](https://github.com/connerohnesorge/conclaude/commit/8f372a33eb95b4b7c476357478447e2463f04884))
- **proposal:** Add-starlight-site-graph (#122) ([260d518](https://github.com/connerohnesorge/conclaude/commit/260d518bfaa510e2e3d89685f541d3e3aac531ec))
- **proposal:** Add-starlight-llms-txt (#120) ([79c39b5](https://github.com/connerohnesorge/conclaude/commit/79c39b51a2e55fa2c299306f9d03575df9c2e00e))
- **proposal:** Add-starlight-links-validator (#119) ([cefc59f](https://github.com/connerohnesorge/conclaude/commit/cefc59f949a98a6647da0a031b315880bc4eff5e))
- **proposal:** Add-starlight-changelogs-plugin (#118) ([5ba8703](https://github.com/connerohnesorge/conclaude/commit/5ba87032835dd8f391b08b0aa4f4940e7307cb23))
- **proposal:** Add-scroll-to-top-plugin (#117) ([1d8bcb6](https://github.com/connerohnesorge/conclaude/commit/1d8bcb6ec0f56880ac96d44f9a240520b435c588))
- **proposal:** Add-starlight-page-actions (#121) ([1591063](https://github.com/connerohnesorge/conclaude/commit/1591063b5a2f8b26dcb18b481d159a8a6dae3c08))
- **archive:** Remove-rounds-feature (#127) ([247647f](https://github.com/connerohnesorge/conclaude/commit/247647f9a17918980fe1c7769fe6e098c690bf14))
- **archive:** Fix-command-working-directory (#126) ([b8da370](https://github.com/connerohnesorge/conclaude/commit/b8da370cb0ce466bf326a685d9fc6fbe9d50fc01))
- **archive:** Add-starlight-star-warp (#134) ([a79958b](https://github.com/connerohnesorge/conclaude/commit/a79958b0b0312a96852165da7b4a1e3086de558b))
- **archive:** Add-git-block-example (#130) ([757df15](https://github.com/connerohnesorge/conclaude/commit/757df155e13604bda7fac91de5af324dc8c1b086))
- **archive:** Replace-placeholder-docs (#133) ([e8adf4c](https://github.com/connerohnesorge/conclaude/commit/e8adf4cb6c19c6b6ecf4ca3e8603cbdcc5d76448))
- **archive:** Add-starlight-llms-txt (#132) ([6d02018](https://github.com/connerohnesorge/conclaude/commit/6d0201876a1e3b068356dbc1ad2c4b5b6735d59a))
- **archive:** Add-starlight-changelogs-plugin (#131) ([ed833ff](https://github.com/connerohnesorge/conclaude/commit/ed833ffd1a28b2eea943de7c8d4ac706b55241ae))
- **proposal:** Add-config-documentation-generator (#128) ([ec4578e](https://github.com/connerohnesorge/conclaude/commit/ec4578e60cb64e3250b79ced85b771ed68b692f9))

## [0.2.3] - 2025-11-29

### Features

- Add agent_id and agent_transcript_path to SubagentStop payload (#80) ([12cb9ee](https://github.com/connerohnesorge/conclaude/commit/12cb9ee15e3f1ec6459d0e4fed166ae9b1e77310))
- Add SubagentStart hook support and tool_use_id field (#83) ([d945cea](https://github.com/connerohnesorge/conclaude/commit/d945cea0eac8fc352216b70a79f23478fa163e87))
- Enhance refine-prevent-root-additions spec delta with complete sections (#87) ([857c615](https://github.com/connerohnesorge/conclaude/commit/857c615f5535dd051fd3a25622ad981c3b750db5))
- [**breaking**] Consolidate rules configuration into preToolUse section (#99) ([a075723](https://github.com/connerohnesorge/conclaude/commit/a075723d3f87ef88d709b2ed1afb0f2b618f5026))
- Add .conclaude.yml and .conclaude.yaml to default uneditableFiles (#102) ([0596d9c](https://github.com/connerohnesorge/conclaude/commit/0596d9c519f27fbbe686796f743994f27da9664f))
- Add preventUpdateGitIgnored setting to protect git-ignored files (#103) ([80a3491](https://github.com/connerohnesorge/conclaude/commit/80a3491ece4b6c15733d9daa859ff102daf479be))
- Implement permission request hook with configuration system (#105) ([4eed513](https://github.com/connerohnesorge/conclaude/commit/4eed5130c864f9600981e303374d06b60e06fd17))
- Implement custom uneditable file messages (#106) ([e84d1ad](https://github.com/connerohnesorge/conclaude/commit/e84d1adfe4e24bb4ada01e1c46e1685222a2efc5))
- Implement subagent stop commands with pattern matching (#107) ([0b93b8e](https://github.com/connerohnesorge/conclaude/commit/0b93b8ec52979e94550b5786b2fa99bdacd0a44a))

### Bug Fixes

- Implement preventAdditions enforcement in preToolUse hook (#100) ([9307ecb](https://github.com/connerohnesorge/conclaude/commit/9307ecb7e72bf9e2b4444f1f4969327fd1ac76d4))
- Read version from Cargo.toml in flake.nix (#101) ([96007b6](https://github.com/connerohnesorge/conclaude/commit/96007b6b3d8e5a76fd809349d00da0fd9f229745))
- Fix spectr specs ([22e0512](https://github.com/connerohnesorge/conclaude/commit/22e05126761b3b0a4070fa44f7beedc40c2be778))
- Fix conclaude spec validation on stop hook to use spectr not openspec ([39c3663](https://github.com/connerohnesorge/conclaude/commit/39c36632493a0dce13fc5d21956b5a577202ba65))
- Make matcher field optional in ClaudeHookMatcher (#108) ([19ae5d3](https://github.com/connerohnesorge/conclaude/commit/19ae5d392c1552479965536940f5432d5bb5ea5e))

### Refactoring

- Rename openspec directory to spectr (#92) ([16be778](https://github.com/connerohnesorge/conclaude/commit/16be778ce8932fedecd40772da093c167b04baac))

### Miscellaneous Tasks

- **release:** Update CHANGELOG.md for v0.2.1 ([ab0a736](https://github.com/connerohnesorge/conclaude/commit/ab0a736af505f851f01f4af08b26b886e2b46e51))
- Archive add-tool-use-id-field spec and update canonical specs (#91) ([246ef32](https://github.com/connerohnesorge/conclaude/commit/246ef32ff35e2d338639f8796f63a08f93186994))
- Archive add-subagent-start-hook change (#88) ([41f8c35](https://github.com/connerohnesorge/conclaude/commit/41f8c355ead6306ec4f5d0248780e87ed9e74590))
- Archive add-subagent-start-hook change and create subagent-hooks spec (#90) ([f4f93c2](https://github.com/connerohnesorge/conclaude/commit/f4f93c247f812a472bb924d04f3d6a61eaad2f41))

## [0.2.1] - 2025-11-13

### Bug Fixes

- Fix changelog workflow to use git-cliff action (#75)

Replace nix-based approach with proven git-cliff GitHub action from official repo. Update cliff.toml template to use remote.github variables instead of env vars. ([a8932e5](https://github.com/connerohnesorge/conclaude/commit/a8932e54839d4c39e977995ff851487690bf65c9))

## [0.2.0] - 2025-11-13

## [0.1.9] - 2025-11-13

## [0.1.8] - 2025-11-12

## [0.1.7] - 2025-11-03

### Features

- Comprehensive updates to configuration, hooks, and testing modules (#48) ([6f5a2be](https://github.com/connerohnesorge/conclaude/commit/6f5a2be63e22da16b78f9c1ef89ba8dfa8d16dd0))
- [**breaking**] Remove gitWorktree configuration support (#53) ([bece5f4](https://github.com/connerohnesorge/conclaude/commit/bece5f47a2dc61efccab3c71be88188f866a2859))
- Comprehensive openspec framework integration and hook reconfiguration (#54) ([922384a](https://github.com/connerohnesorge/conclaude/commit/922384a39aa7996308612193881938754ed5aa51))
- Replace logging system with notification system and add command timeout (#58) ([e2fca5f](https://github.com/connerohnesorge/conclaude/commit/e2fca5fa87dd79589858fc0e2dac6526422dafda))

### Documentation

- Enhance README.md with comprehensive installation guide and version update ([16464b7](https://github.com/connerohnesorge/conclaude/commit/16464b74e56138116596ec8eded4fb223c6d3c78))

## [0.1.6] - 2025-10-08

## [0.1.5] - 2025-10-08

## [0.1.4] - 2025-10-08

### Features

- Migrate to cargo-dist and add SessionEnd hook (#46) ([59f39b1](https://github.com/connerohnesorge/conclaude/commit/59f39b12e20f83332e1a7d49d0de1780cdd296b2))

## [0.1.3] - 2025-10-06

### Bug Fixes

- Resolve GitHub Actions workflow failures ([3074abd](https://github.com/connerohnesorge/conclaude/commit/3074abdeef28b46bfbad8422d77984e26987cac3))
- Escape remaining backticks in schema.yml workflow ([615d1cd](https://github.com/connerohnesorge/conclaude/commit/615d1cd6f99893ea822d2862dce6633aa254c1a0))
- Rewrite Create summary step using heredoc ([5da3742](https://github.com/connerohnesorge/conclaude/commit/5da37427a690e5555df80b5eefa851f870ac42e5))
- Make permissions field optional in ClaudeSettings ([1a5565c](https://github.com/connerohnesorge/conclaude/commit/1a5565c5b0fe3a54e5b8bca8062c27f7e8ad5b2c))
- Make hooks field optional in ClaudeSettings ([21b774b](https://github.com/connerohnesorge/conclaude/commit/21b774b0df85f6b4453bb3e7d894e6df9891c390))
- Prevent includeCoAuthoredBy from serializing as null ([379152c](https://github.com/connerohnesorge/conclaude/commit/379152c7307bfa8589175caa58c942a2914dccca))
- Fix grep stop config hooks having default values and not being checked if empty/set to null ([677fa63](https://github.com/connerohnesorge/conclaude/commit/677fa63c43a304ffb1f4628a48931ab55309b071))

### Testing

- Trigger workflow testing ([9768d8b](https://github.com/connerohnesorge/conclaude/commit/9768d8b8817e3bd9355831347c9a8af7cb19ea6a))
- Trigger Auto-update JSON Schema workflow ([c26d842](https://github.com/connerohnesorge/conclaude/commit/c26d8423448209ca3918408cc4e4e6e7cd1001a3))
- Verify final workflow fixes ([c5f8a44](https://github.com/connerohnesorge/conclaude/commit/c5f8a44eaadf7fc9d2c2ae4ac53df918a3f979db))
- Final verification of workflow fixes ([629f509](https://github.com/connerohnesorge/conclaude/commit/629f509605998715493904b8ef2c2ac58dfe8bb5))

### Miscellaneous Tasks

- Auto-update JSON schema ([54970ef](https://github.com/connerohnesorge/conclaude/commit/54970efb47fe63ba1f18e36c78d036b7e99d1c4a))
- Nix-powered CI and release; remove schema auto-update (#31) ([30d4b87](https://github.com/connerohnesorge/conclaude/commit/30d4b87b2a559b9e503fab79a1d27cb61a87778e))
- Ci/nix actions (#32)

* ci: add Nix-powered CI and release; remove schema auto-update workflows

* docs(README): add CI/Release badges and Releases section with download instructions

* fix .claude/settings.json parser ([8a7b8ac](https://github.com/connerohnesorge/conclaude/commit/8a7b8ace54296d5d7df295b2f3a1aca73d9744bb))

### Fix

- Handle empty grep rules arrays correctly (#35) ([df5660d](https://github.com/connerohnesorge/conclaude/commit/df5660dc26713e42358ce802b9e25aebb5c565cd))

## [0.1.0] - 2025-09-05

### Bug Fixes

- Fix dev dep for types of yargs ([2dec02c](https://github.com/connerohnesorge/conclaude/commit/2dec02c11ebda64568f0ece427d24b65d7e9b3b4))
- Fix bin packaging ([c39dc29](https://github.com/connerohnesorge/conclaude/commit/c39dc29e33673c7d7e15c1f0902daa28f79d4a46))
- Fix package.json merge conflicts ([a5cf069](https://github.com/connerohnesorge/conclaude/commit/a5cf0699765088333915c99f3bdebf8485294a8f))
- Fix removal of package-lock.json ([6914d3d](https://github.com/connerohnesorge/conclaude/commit/6914d3daf292c82408496f82cd70b857e42a7daf))
- Fix npm hash for nix packaging ([5bd751f](https://github.com/connerohnesorge/conclaude/commit/5bd751faa8e5a5bafc8b9a8e87290e76c2f8bda8))
- Fix infinite mode ([110f472](https://github.com/connerohnesorge/conclaude/commit/110f4722c9c042ead0e6bdd92bf0cdd2ba54d763))

<!-- generated by git-cliff -->

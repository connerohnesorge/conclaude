## 1. Implementation

- [ ] 1.1 Add `show_command` field to `StopCommand` struct in `src/config.rs` with `#[serde(default = "default_true", rename = "showCommand")]`
- [ ] 1.2 Add `show_command` field to `SubagentStopCommand` struct in `src/config.rs` with same attributes
- [ ] 1.3 Add `default_true` helper function returning `Some(true)` if not already present
- [ ] 1.4 Update `StopCommandConfig` struct in `src/hooks.rs` to include `show_command: bool` field
- [ ] 1.5 Update `SubagentStopCommandConfig` struct in `src/hooks.rs` to include `show_command: bool` field
- [ ] 1.6 Update `build_stop_commands` function to propagate `show_command` value (default to `true`)
- [ ] 1.7 Update `build_subagent_stop_commands` function to propagate `show_command` value (default to `true`)
- [ ] 1.8 Conditionally print "Executing command X/Y: <command>" based on `show_command` in `execute_stop_commands`
- [ ] 1.9 Conditionally print command execution line in `execute_subagent_stop_commands`

## 2. Schema Update

- [ ] 2.1 Regenerate `conclaude-schema.json` with `cargo run -- generate-schema`

## 3. Testing

- [ ] 3.1 Add unit test: config parsing with `showCommand: true` explicit
- [ ] 3.2 Add unit test: config parsing with `showCommand: false`
- [ ] 3.3 Add unit test: config parsing without `showCommand` (verify default `true`)
- [ ] 3.4 Add integration test: verify command line is printed when `showCommand: true`
- [ ] 3.5 Add integration test: verify command line is suppressed when `showCommand: false`

## 4. Documentation

- [ ] 4.1 Update `.conclaude.yaml` comments with `showCommand` option
- [ ] 4.2 Update README.md command configuration section

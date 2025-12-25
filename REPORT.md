# Test Removal Candidate Report

## Candidates

### src/types.rs
- test_hook_result_with_context (line ~294): only checks a trivial constructor assigns fields; no branching or edge cases.
- test_pre_tool_use_payload_serialization_with_tool_use_id (line ~305) and test_post_tool_use_payload_serialization_with_tool_use_id (line ~345): redundant with test_tool_use_id_round_trip_serialization (line ~387), which already serializes and deserializes tool_use_id.

### src/config_test.rs
- test_uneditable_file_rule_pattern_extraction (line ~479), test_uneditable_file_rule_message_extraction (line ~493), test_uneditable_file_rule_agent_extraction (line ~514): only mirror enum accessors; behavior is already exercised by YAML parsing tests that verify pattern/message/agent values.

### src/hooks_tests.rs (unit tests in src)
- test_bash_command_prefix_match_at_start (line ~228), test_bash_command_prefix_no_match_middle (line ~245), test_bash_command_wildcard_variations (line ~261): these validate glob crate behavior or re-implement prefix matching logic without exercising production code.

### tests/types_tests.rs
- test_validate_subagent_stop_payload_different_agent_types (line ~291) and test_validate_subagent_start_payload_different_agent_types (line ~623): validation only checks non-empty strings, so extra agent labels do not add coverage.
- test_validate_subagent_stop_payload_multiple_missing_fields (line ~312): redundant with the single-field missing tests immediately above it.
- test_subagent_start_payload_json_round_trip_coder/tester/stuck (lines ~712/733/754): redundant with test_subagent_start_payload_deserialize_from_valid_json plus test_subagent_start_payload_serialization_round_trip.

### tests/hooks_tests.rs
- Bash validation block (lines ~343-817): test_bash_validation_* cases re-implement matching logic using glob patterns instead of exercising check_tool_usage_rules, so they largely validate test-local logic.
- SubagentStop payload structure/variants (lines ~918-1113): test_subagent_stop_payload_structure, test_subagent_stop_payload_with_different_agent_ids, test_subagent_stop_payload_json_round_trip_*, test_subagent_stop_payload_with_long_paths, test_subagent_stop_payload_with_special_characters_in_paths, test_subagent_stop_payload_agent_id_case_sensitive, test_subagent_stop_payload_with_empty_permission_mode, test_subagent_stop_payload_stop_hook_active_true_and_false. These mostly assert locally constructed data or trivial validation and overlap with tests/types_tests.rs.
- Environment variable simulations (lines ~1169-1264): test_subagent_stop_environment_variable_setting_simulation, test_subagent_stop_environment_variables_different_agents, test_subagent_stop_environment_variables_special_paths, test_subagent_stop_multiple_sequential_invocations. These set/read env vars without calling production code.
- Validation error message tests (lines ~1333-1368): duplicates with tests/types_tests.rs that already validate error messages for missing fields.
- Root/preventAdditions documentation tests (lines ~1480-1516, 1664, 1730, 1937, 2130): test_prevent_root_additions_existing_config_files, test_prevent_root_additions_write_vs_edit_tool, test_prevent_additions_only_affects_write_tool, test_prevent_additions_empty_array_allows_all, test_prevent_additions_expected_error_message_format, test_prevent_additions_does_not_affect_edit_operations. These do not exercise the enforcement path and mainly check obvious string/tool-name facts.
- test_prevent_additions_write_tool_with_various_paths (line ~2044): redundant with earlier extract_file_path tests.

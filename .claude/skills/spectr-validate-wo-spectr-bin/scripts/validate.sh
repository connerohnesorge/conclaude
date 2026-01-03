#!/usr/bin/env bash
#
# validate.sh - Validate Spectr specifications and change proposals
#
# This script provides validation capabilities equivalent to `spectr validate`
# for environments where the spectr binary is not available (sandboxed AI
# environments, CI pipelines, fresh checkouts).
#
# Usage:
#   validate.sh --spec <spec-id>      # Validate single spec
#   validate.sh --change <change-id>  # Validate single change
#   validate.sh --all                 # Validate all specs and changes
#   validate.sh --json                # Output JSON (combine with above)
#
# Exit codes:
#   0 - All validations passed
#   1 - One or more validations failed
#   2 - Usage error (invalid arguments)
#
# Environment variables:
#   SPECTR_DIR - Custom spectr directory location (default: "spectr")
#

# Task 2.2: Add set flags for robust error handling
set -euo pipefail

# ============================================================================
# Task 2.3: Configuration section (SPECTR_DIR environment variable support)
# ============================================================================

readonly SCRIPT_VERSION="1.0.0"
readonly SPECTR_DIR="${SPECTR_DIR:-spectr}"
readonly SPECS_DIR="$SPECTR_DIR/specs"
readonly CHANGES_DIR="$SPECTR_DIR/changes"

# ============================================================================
# Task 2.4: Color setup function with TTY detection
# ============================================================================

# Color codes for terminal output (ANSI codes only if interactive)
# These are set dynamically based on whether stdout is a TTY
RED=""
YELLOW=""
GREEN=""
NC=""  # No Color

# setup_colors - Configure ANSI color codes for terminal output
#
# Detects whether stdout is a terminal (TTY) and enables color codes
# accordingly. When output is piped or redirected, colors are disabled
# to ensure clean output in logs and files.
#
# This function is called once at script initialization to set global
# color variables used throughout the script.
#
# Globals set:
#   RED    - ANSI code for red text (errors)
#   YELLOW - ANSI code for yellow text (warnings)
#   GREEN  - ANSI code for green text (success)
#   NC     - ANSI code to reset color (no color)
setup_colors() {
    # Only enable colors if stdout is a TTY (interactive terminal)
    # This check uses [[ -t 1 ]] which tests if file descriptor 1 (stdout) is a terminal
    if [[ -t 1 ]]; then
        RED='\033[0;31m'
        YELLOW='\033[0;33m'
        GREEN='\033[0;32m'
        NC='\033[0m'
    fi
}

# Initialize colors at script startup
setup_colors

# ============================================================================
# Task 2.5: Define regex patterns matching internal/markdown/ (Requirements, Requirement, Scenario headers)
# ============================================================================
#
# These regex patterns MUST stay synchronized with the Go implementation in
# internal/markdown/compat.go to ensure consistent validation behavior.
#
# Pattern design:
# - Use [[:space:]] instead of \s for POSIX compatibility
# - Use + for one-or-more, * for zero-or-more
# - Use (.+) to capture groups for extraction
# - Use $ to anchor end of line and prevent partial matches

# Section headers - matches internal/markdown/compat.go:MatchH2SectionHeader
# Pattern matches "## Requirements" with flexible whitespace
# Example: "## Requirements", "##  Requirements  "
readonly REQUIREMENTS_SECTION_PATTERN='^##[[:space:]]+Requirements[[:space:]]*$'

# Requirement header - matches internal/markdown/compat.go:MatchRequirementHeader
# Pattern: "### Requirement: Name" with flexible whitespace
# Captures the requirement name after "Requirement:" for error reporting
# Example: "### Requirement: User Authentication" -> captures "User Authentication"
readonly REQUIREMENT_HEADER_PATTERN='^###[[:space:]]+Requirement:[[:space:]]+(.+)$'

# Scenario header - matches internal/markdown/compat.go:MatchScenarioHeader
# Pattern: "#### Scenario: Name" with flexible whitespace
# Example: "#### Scenario: Valid login", "####  Scenario: Invalid password"
readonly SCENARIO_HEADER_PATTERN='^####[[:space:]]+Scenario:'

# ============================================================================
# Task 2.6: Define malformed scenario patterns (3/5/6 hashtags, bold, bullets)
# ============================================================================
#
# Common user errors when writing scenarios:
# - Using wrong header level (### or ##### instead of ####)
# - Using bold formatting (**Scenario:**) instead of headers
# - Using bullet points (- **Scenario:**) instead of headers
#
# These patterns detect malformed scenarios to provide helpful error messages

# Malformed scenario patterns - detect incorrect formatting
# These should be "#### Scenario:" but are incorrectly formatted
readonly MALFORMED_SCENARIO_3_HASH='^###[[:space:]]+Scenario:'     # Wrong: ### Scenario
readonly MALFORMED_SCENARIO_5_HASH='^#####[[:space:]]+Scenario:'   # Wrong: ##### Scenario
readonly MALFORMED_SCENARIO_6_HASH='^######[[:space:]]+Scenario:'  # Wrong: ###### Scenario
readonly MALFORMED_SCENARIO_BOLD='^\*\*Scenario:'                  # Wrong: **Scenario:**
readonly MALFORMED_SCENARIO_BULLET_BOLD='^-[[:space:]]+\*\*Scenario:' # Wrong: - **Scenario:**

# ============================================================================
# Task 2.7: Define delta section patterns (ADDED, MODIFIED, REMOVED, RENAMED)
# ============================================================================
#
# Change proposals use delta sections to specify what changed:
# - ADDED: New requirements being added to a specification
# - MODIFIED: Existing requirements being changed
# - REMOVED: Requirements being removed from a specification
# - RENAMED: Requirements being renamed (special FROM/TO format)
#
# Each delta type has different validation rules (see flush_delta_requirement)

# Delta section headers - matches internal/markdown/compat.go:MatchH2DeltaSection
# Pattern: "## ADDED Requirements", "## MODIFIED Requirements", etc.
# Case-insensitive on the delta type to support both "ADDED" and "Added"
readonly DELTA_ADDED_PATTERN='^##[[:space:]]+(ADDED|Added)[[:space:]]+Requirements[[:space:]]*$'
readonly DELTA_MODIFIED_PATTERN='^##[[:space:]]+(MODIFIED|Modified)[[:space:]]+Requirements[[:space:]]*$'
readonly DELTA_REMOVED_PATTERN='^##[[:space:]]+(REMOVED|Removed)[[:space:]]+Requirements[[:space:]]*$'
readonly DELTA_RENAMED_PATTERN='^##[[:space:]]+(RENAMED|Renamed)[[:space:]]+Requirements[[:space:]]*$'

# ============================================================================
# Task 2.8: Define task item pattern (flexible format with optional IDs)
# ============================================================================
#
# Task files can use various formats for task items:
# - With or without task IDs (e.g., "1.1", "2.3")
# - Lowercase or uppercase completion markers (x or X)
# - Flexible whitespace around brackets and dashes
#
# This pattern accepts all valid formats to avoid false positives

# Task item pattern - matches internal/markdown/compat.go:MatchFlexibleTask
# Flexible format supporting:
#   - [ ] Task                             (uncompleted, no ID)
#   - [x] Completed task                   (completed, no ID)
#   - [ ] 1.1 Numbered task                (uncompleted, with ID)
#   - [X] 2.3 Completed numbered task      (completed, uppercase X, with ID)
readonly TASK_ITEM_PATTERN='^[[:space:]]*-[[:space:]]+\[([ xX])\][[:space:]]+'

# ============================================================================
# Task 2.9: Implement issue collection arrays and counters
# ============================================================================

# Issue collection arrays and counters
# ISSUES array stores all validation issues in format: "level|file|line|message"
declare -a ISSUES=()

# ISSUE_COUNTS associative array tracks error and warning counts
declare -A ISSUE_COUNTS=([errors]=0 [warnings]=0)

# Validated items tracking for reporting
declare -a VALIDATED_ITEMS=()
declare -A ITEM_TYPES=()

# ============================================================================
# Task 2.10: Implement add_issue() function for standardized issue tracking
# ============================================================================

# add_issue adds a validation issue to the global ISSUES array
# and increments the appropriate counter
#
# Arguments:
#   $1 - level: "ERROR" or "WARNING"
#   $2 - file: file path or identifier
#   $3 - line: line number
#   $4 - message: error message
#
# Example:
#   add_issue "ERROR" "specs/auth/spec.md" 15 "Missing scenario"
add_issue() {
    local level="$1"
    local file="$2"
    local line="$3"
    local message="$4"

    # Store issue in pipe-delimited format for easy parsing
    ISSUES+=("${level}|${file}|${line}|${message}")

    # Increment appropriate counter
    case "$level" in
        ERROR)
            ((++ISSUE_COUNTS[errors]))
            ;;
        WARNING)
            ((++ISSUE_COUNTS[warnings]))
            ;;
    esac
}

# ============================================================================
# Section 3: Spec File Validation (Tasks 3.1-3.10)
# ============================================================================

# Task 3.7: check_malformed_scenario - Detect malformed scenario formatting
#
# Checks if a line contains a malformed scenario pattern (wrong header levels,
# bullets, bold formatting, etc.)
#
# Arguments:
#   $1 - file: file path for error reporting
#   $2 - line_num: current line number
#   $3 - line: the line content to check
check_malformed_scenario() {
    local file="$1"
    local line_num="$2"
    local line="$3"

    # Check for 3 hashtags (### Scenario:)
    if [[ "$line" =~ $MALFORMED_SCENARIO_3_HASH ]]; then
        add_issue "ERROR" "$file" "$line_num" \
            "Scenarios must use '#### Scenario:' format (4 hashtags followed by 'Scenario:')"
        return
    fi

    # Check for 5 hashtags (##### Scenario:)
    if [[ "$line" =~ $MALFORMED_SCENARIO_5_HASH ]]; then
        add_issue "ERROR" "$file" "$line_num" \
            "Scenarios must use '#### Scenario:' format (4 hashtags followed by 'Scenario:')"
        return
    fi

    # Check for 6 hashtags (###### Scenario:)
    if [[ "$line" =~ $MALFORMED_SCENARIO_6_HASH ]]; then
        add_issue "ERROR" "$file" "$line_num" \
            "Scenarios must use '#### Scenario:' format (4 hashtags followed by 'Scenario:')"
        return
    fi

    # Check for bold formatting (**Scenario:)
    if [[ "$line" =~ $MALFORMED_SCENARIO_BOLD ]]; then
        add_issue "ERROR" "$file" "$line_num" \
            "Scenarios must use '#### Scenario:' format (4 hashtags followed by 'Scenario:')"
        return
    fi

    # Check for bullet + bold (- **Scenario:)
    if [[ "$line" =~ $MALFORMED_SCENARIO_BULLET_BOLD ]]; then
        add_issue "ERROR" "$file" "$line_num" \
            "Scenarios must use '#### Scenario:' format (4 hashtags followed by 'Scenario:')"
        return
    fi
}

# Task 3.8: validate_requirement_state - Validate requirement state when flushing
#
# Validates that a requirement has proper SHALL/MUST keywords and at least one scenario.
# This is called when transitioning between requirements or sections.
#
# Arguments:
#   $1 - file: file path for error reporting
#   $2 - requirement_name: name of the requirement
#   $3 - requirement_line: line number where requirement started
#   $4 - has_scenario: "true" if requirement has scenarios, "false" otherwise
#   $5 - has_shall_must: "true" if requirement has SHALL/MUST, "false" otherwise
validate_requirement_state() {
    local file="$1"
    local requirement_name="$2"
    local requirement_line="$3"
    local has_scenario="$4"
    local has_shall_must="$5"

    # Skip validation if requirement_name is empty (no requirement to validate)
    if [[ -z "$requirement_name" ]]; then
        return
    fi

    # Build requirement path for clearer error messages
    local req_path="${file}: Requirement '${requirement_name}'"

    # Task 3.6: Check for SHALL/MUST keywords (ERROR in strict mode)
    if [[ "$has_shall_must" != "true" ]]; then
        add_issue "ERROR" "$req_path" "$requirement_line" \
            "Requirement should contain SHALL or MUST to indicate normative requirement"
    fi

    # Task 3.5 (validation): Check for at least one scenario (ERROR in strict mode)
    if [[ "$has_scenario" != "true" ]]; then
        add_issue "ERROR" "$req_path" "$requirement_line" \
            "Requirement should have at least one scenario"
    fi
}

# Task 3.1: validate_spec_file - Main spec file validation function
#
# Validates a spec file by reading line-by-line, tracking state, and validating
# requirements according to Spectr rules.
#
# Arguments:
#   $1 - file: path to the spec.md file to validate
#
# State tracking:
#   - in_requirements: whether we're currently in the ## Requirements section
#   - current_requirement: name of the current requirement being processed
#   - requirement_line: line number where current requirement started
#   - has_scenario: whether current requirement has any scenarios
#   - has_shall_must: whether current requirement contains SHALL or MUST
#   - found_requirements_section: whether we found a ## Requirements section
validate_spec_file() {
    local file="$1"

    # Task 3.2: Implement state tracking variables
    local line_num=0
    local in_requirements=false
    local current_requirement=""
    local requirement_line=0
    local has_scenario=false
    local has_shall_must=false
    local found_requirements_section=false

    # Task 3.1: Line-by-line reading
    # Read file line by line, handling files without trailing newline
    # The || [[ -n "$line" ]] ensures we process the last line even if it lacks a newline
    while IFS= read -r line || [[ -n "$line" ]]; do
        ((++line_num))

        # ========================================================================
        # State Machine: Section Detection
        # ========================================================================
        # The spec file parser is a state machine with two main states:
        # - in_requirements=true: Currently processing inside ## Requirements section
        # - in_requirements=false: Outside Requirements section (skip processing)

        # Task 3.3: Detect ## Requirements section
        if [[ "$line" =~ $REQUIREMENTS_SECTION_PATTERN ]]; then
            # Task 3.9: Flush current requirement before entering new section
            # This ensures we validate the last requirement from a previous section
            if [[ -n "$current_requirement" ]]; then
                validate_requirement_state "$file" "$current_requirement" \
                    "$requirement_line" "$has_scenario" "$has_shall_must"
                current_requirement=""
            fi

            found_requirements_section=true
            in_requirements=true
            continue
        fi

        # Detect other ## sections (exit Requirements section)
        # Any ## header that's not "## Requirements" ends the Requirements section
        if [[ "$line" =~ ^##[[:space:]] ]]; then
            # Task 3.9: Flush current requirement before leaving section
            # This validates the last requirement in the Requirements section
            if [[ -n "$current_requirement" ]]; then
                validate_requirement_state "$file" "$current_requirement" \
                    "$requirement_line" "$has_scenario" "$has_shall_must"
                current_requirement=""
            fi

            in_requirements=false
            continue
        fi

        # Only process lines within Requirements section
        # Lines outside ## Requirements are ignored (no validation needed)
        if ! $in_requirements; then
            continue
        fi

        # ========================================================================
        # State Machine: Requirement and Scenario Detection
        # ========================================================================
        # Within Requirements section, track current requirement and its properties

        # Task 3.4: Detect ### Requirement: header
        if [[ "$line" =~ $REQUIREMENT_HEADER_PATTERN ]]; then
            # Task 3.9: Flush previous requirement before starting new one
            # Each new ### Requirement: header ends the previous requirement
            if [[ -n "$current_requirement" ]]; then
                validate_requirement_state "$file" "$current_requirement" \
                    "$requirement_line" "$has_scenario" "$has_shall_must"
            fi

            # Start tracking new requirement
            # BASH_REMATCH[1] contains the captured requirement name from the pattern
            current_requirement="${BASH_REMATCH[1]}"
            requirement_line=$line_num
            has_scenario=false
            has_shall_must=false
            continue
        fi

        # Task 3.5: Detect #### Scenario: header
        if [[ "$line" =~ $SCENARIO_HEADER_PATTERN ]]; then
            # Set flag to true - requirement has at least one scenario
            has_scenario=true
            continue
        fi

        # ========================================================================
        # Content Validation: SHALL/MUST keywords and malformed scenarios
        # ========================================================================
        # Only check content within a requirement (current_requirement is set)

        # Task 3.6: Detect SHALL/MUST keywords in requirement content
        # Case-insensitive match for SHALL or MUST (as whole words)
        if [[ -n "$current_requirement" ]]; then
            if [[ "$line" =~ [Ss][Hh][Aa][Ll][Ll]|[Mm][Uu][Ss][Tt] ]]; then
                has_shall_must=true
            fi

            # Task 3.7: Check for malformed scenarios
            # Detect common formatting errors like ### Scenario: or **Scenario:**
            check_malformed_scenario "$file" "$line_num" "$line"
        fi
    done < "$file"

    # Task 3.9: Flush final requirement at end of file
    if [[ -n "$current_requirement" ]]; then
        validate_requirement_state "$file" "$current_requirement" \
            "$requirement_line" "$has_scenario" "$has_shall_must"
    fi

    # Task 3.10: Report error if Requirements section is missing
    if ! $found_requirements_section; then
        add_issue "ERROR" "$file" 1 \
            "Missing required '## Requirements' section"
    fi
}

# ============================================================================
# Section 4: Change Delta Validation (Tasks 4.1-4.12)
# ============================================================================

# flush_delta_requirement - Flush and validate a delta requirement
#
# Task 4.8: Implement flush_delta_requirement() function with section-specific rules
#
# Validates a delta requirement based on its delta type:
# - ADDED/MODIFIED: Must have scenario and SHALL/MUST
# - REMOVED: Skip scenario/SHALL validation (Task 4.11)
# - RENAMED: Skip normal validation (different format, Task 4.12)
#
# Arguments:
#   $1 - file: file path for error reporting
#   $2 - delta_type: ADDED, MODIFIED, REMOVED, or RENAMED
#   $3 - requirement_name: name of the requirement
#   $4 - requirement_line: line number where requirement started
#   $5 - has_scenario: "true" if requirement has scenarios, "false" otherwise
#   $6 - has_shall_must: "true" if requirement has SHALL/MUST, "false" otherwise
flush_delta_requirement() {
    local file="$1"
    local delta_type="$2"
    local requirement_name="$3"
    local requirement_line="$4"
    local has_scenario="$5"
    local has_shall_must="$6"

    # Skip validation if requirement_name is empty (no requirement to validate)
    if [[ -z "$requirement_name" ]]; then
        return
    fi

    # Build requirement path for clearer error messages
    local req_path="${file}: ${delta_type} Requirement '${requirement_name}'"

    # Task 4.11: Skip scenario/SHALL validation for REMOVED requirements
    if [[ "$delta_type" == "REMOVED" ]]; then
        # REMOVED requirements don't need scenarios or SHALL/MUST
        return
    fi

    # Task 4.12: Skip normal validation for RENAMED section (different format)
    if [[ "$delta_type" == "RENAMED" ]]; then
        # RENAMED requirements use FROM/TO format, validated separately
        return
    fi

    # Task 4.6: Implement delta requirement validation (scenarios, SHALL/MUST for ADDED/MODIFIED)
    # For ADDED and MODIFIED requirements, validate scenarios and SHALL/MUST

    # Check for SHALL/MUST keywords (ERROR)
    if [[ "$has_shall_must" != "true" ]]; then
        add_issue "ERROR" "$req_path" "$requirement_line" \
            "${delta_type} requirement must contain SHALL or MUST"
    fi

    # Check for at least one scenario (ERROR)
    if [[ "$has_scenario" != "true" ]]; then
        add_issue "ERROR" "$req_path" "$requirement_line" \
            "${delta_type} requirement must have at least one scenario"
    fi
}

# flush_delta_section - Flush and validate a delta section
#
# Task 4.9: Implement flush_delta_section() function for empty section checks
#
# Validates that a delta section is not empty (has at least one requirement).
#
# Arguments:
#   $1 - file: file path for error reporting
#   $2 - delta_type: ADDED, MODIFIED, REMOVED, or RENAMED
#   $3 - section_line: line number where section started
#   $4 - requirement_count: number of requirements in this section
flush_delta_section() {
    local file="$1"
    local delta_type="$2"
    local section_line="$3"
    local requirement_count="$4"

    # Task 4.7: Implement empty delta section detection
    # Only validate if we're actually in a delta section (delta_type is non-empty)
    if [[ -n "$delta_type" ]] && [[ "$requirement_count" -eq 0 ]]; then
        add_issue "ERROR" "$file" "$section_line" \
            "${delta_type} Requirements section is empty (no requirements found)"
    fi
}

# validate_delta_spec_file - Validate a single delta spec file
#
# Task 4.3: Implement validate_delta_spec_file() function with delta counting
#
# Validates a delta spec file by reading line-by-line, tracking delta sections,
# and validating requirements according to their delta type.
#
# Arguments:
#   $1 - file: path to the spec.md delta file to validate
#
# State tracking:
#   - current_delta_type: ADDED, MODIFIED, REMOVED, RENAMED, or empty
#   - delta_section_line: line number where current delta section started
#   - delta_requirement_count: number of requirements in current delta section
#   - current_requirement: name of the current requirement being processed
#   - requirement_line: line number where current requirement started
#   - has_scenario: whether current requirement has any scenarios
#   - has_shall_must: whether current requirement contains SHALL or MUST
#   - total_delta_count: total number of delta sections found in file
validate_delta_spec_file() {
    local file="$1"

    # State tracking variables
    local line_num=0
    local current_delta_type=""
    local delta_section_line=0
    local delta_requirement_count=0
    local current_requirement=""
    local requirement_line=0
    local has_scenario=false
    local has_shall_must=false
    local total_delta_count=0

    # Task 4.3: Line-by-line reading with delta counting
    # Read file line by line, handling files without trailing newline
    while IFS= read -r line || [[ -n "$line" ]]; do
        ((++line_num))

        # ========================================================================
        # State Machine: Delta Section Detection
        # ========================================================================
        # Delta spec files have multiple delta sections (ADDED, MODIFIED, etc.)
        # Each section has different validation rules based on delta type
        # State is tracked via current_delta_type variable

        # Task 4.4: Implement delta section detection (ADDED/MODIFIED/REMOVED/RENAMED Requirements)
        # Check for ADDED Requirements
        if [[ "$line" =~ $DELTA_ADDED_PATTERN ]]; then
            # Flush previous requirement and section before starting new section
            # This validates the last requirement from the previous section
            # and checks if the previous section was empty
            flush_delta_requirement "$file" "$current_delta_type" "$current_requirement" \
                "$requirement_line" "$has_scenario" "$has_shall_must"
            flush_delta_section "$file" "$current_delta_type" "$delta_section_line" \
                "$delta_requirement_count"

            # Start new ADDED section - reset all tracking variables
            current_delta_type="ADDED"
            delta_section_line=$line_num
            delta_requirement_count=0
            current_requirement=""
            ((++total_delta_count))
            continue
        fi

        # Check for MODIFIED Requirements
        if [[ "$line" =~ $DELTA_MODIFIED_PATTERN ]]; then
            # Flush previous requirement and section
            flush_delta_requirement "$file" "$current_delta_type" "$current_requirement" \
                "$requirement_line" "$has_scenario" "$has_shall_must"
            flush_delta_section "$file" "$current_delta_type" "$delta_section_line" \
                "$delta_requirement_count"

            # Start new MODIFIED section
            current_delta_type="MODIFIED"
            delta_section_line=$line_num
            delta_requirement_count=0
            current_requirement=""
            ((++total_delta_count))
            continue
        fi

        # Check for REMOVED Requirements
        if [[ "$line" =~ $DELTA_REMOVED_PATTERN ]]; then
            # Flush previous requirement and section
            flush_delta_requirement "$file" "$current_delta_type" "$current_requirement" \
                "$requirement_line" "$has_scenario" "$has_shall_must"
            flush_delta_section "$file" "$current_delta_type" "$delta_section_line" \
                "$delta_requirement_count"

            # Start new REMOVED section
            current_delta_type="REMOVED"
            delta_section_line=$line_num
            delta_requirement_count=0
            current_requirement=""
            ((++total_delta_count))
            continue
        fi

        # Check for RENAMED Requirements
        if [[ "$line" =~ $DELTA_RENAMED_PATTERN ]]; then
            # Flush previous requirement and section
            flush_delta_requirement "$file" "$current_delta_type" "$current_requirement" \
                "$requirement_line" "$has_scenario" "$has_shall_must"
            flush_delta_section "$file" "$current_delta_type" "$delta_section_line" \
                "$delta_requirement_count"

            # Start new RENAMED section
            current_delta_type="RENAMED"
            delta_section_line=$line_num
            delta_requirement_count=0
            current_requirement=""
            ((++total_delta_count))
            continue
        fi

        # Detect other ## sections (exit delta section)
        # Any ## header that's not a delta section header ends the current delta section
        if [[ "$line" =~ ^##[[:space:]] ]] && [[ -n "$current_delta_type" ]]; then
            # Flush current requirement and section before leaving
            flush_delta_requirement "$file" "$current_delta_type" "$current_requirement" \
                "$requirement_line" "$has_scenario" "$has_shall_must"
            flush_delta_section "$file" "$current_delta_type" "$delta_section_line" \
                "$delta_requirement_count"

            # Clear delta state - we're outside any delta section now
            current_delta_type=""
            current_requirement=""
            continue
        fi

        # Only process lines within a delta section
        # Lines outside delta sections are ignored (no validation needed)
        if [[ -z "$current_delta_type" ]]; then
            continue
        fi

        # ========================================================================
        # State Machine: Requirement Parsing Within Delta Sections
        # ========================================================================
        # Different delta types have different parsing rules:
        # - ADDED/MODIFIED/REMOVED: Parse ### Requirement: headers normally
        # - RENAMED: Special FROM/TO format, different parsing logic

        # Task 4.5: Implement requirement parsing within delta sections
        # Task 4.12: Skip normal requirement parsing for RENAMED section
        if [[ "$current_delta_type" != "RENAMED" ]]; then
            # Normal requirement parsing for ADDED/MODIFIED/REMOVED sections

            # Detect ### Requirement: header
            if [[ "$line" =~ $REQUIREMENT_HEADER_PATTERN ]]; then
                # Flush previous requirement before starting new one
                flush_delta_requirement "$file" "$current_delta_type" "$current_requirement" \
                    "$requirement_line" "$has_scenario" "$has_shall_must"

                # Start tracking new requirement
                # BASH_REMATCH[1] contains the captured requirement name
                current_requirement="${BASH_REMATCH[1]}"
                requirement_line=$line_num
                has_scenario=false
                has_shall_must=false
                ((++delta_requirement_count))
                continue
            fi

            # Detect #### Scenario: header
            if [[ "$line" =~ $SCENARIO_HEADER_PATTERN ]]; then
                has_scenario=true
                continue
            fi

            # Detect SHALL/MUST keywords in requirement content
            # Case-insensitive match for SHALL or MUST (as whole words)
            if [[ -n "$current_requirement" ]]; then
                if [[ "$line" =~ [Ss][Hh][Aa][Ll][Ll]|[Mm][Uu][Ss][Tt] ]]; then
                    has_shall_must=true
                fi

                # Check for malformed scenarios
                check_malformed_scenario "$file" "$line_num" "$line"
            fi
        else
            # Task 4.12: For RENAMED section, just count FROM/TO pairs
            # RENAMED section uses a different format:
            #   - FROM: ### Requirement: Old Name
            #   - TO: ### Requirement: New Name
            # We don't parse individual requirements the same way
            # Instead, we look for FROM: patterns to count renames
            if [[ "$line" =~ FROM:[[:space:]]*###[[:space:]]+Requirement: ]] || \
               [[ "$line" =~ -[[:space:]]*FROM:[[:space:]]*###[[:space:]]+Requirement: ]] || \
               [[ "$line" =~ -[[:space:]]*\`FROM:[[:space:]]*###[[:space:]]+Requirement: ]]; then
                ((++delta_requirement_count))
            fi
        fi
    done < "$file"

    # Flush final requirement and section at end of file
    flush_delta_requirement "$file" "$current_delta_type" "$current_requirement" \
        "$requirement_line" "$has_scenario" "$has_shall_must"
    flush_delta_section "$file" "$current_delta_type" "$delta_section_line" \
        "$delta_requirement_count"

    # Task 4.10: Implement total delta count check (error if zero deltas)
    if [[ "$total_delta_count" -eq 0 ]]; then
        add_issue "ERROR" "$file" 1 \
            "Change must have at least one delta (ADDED, MODIFIED, REMOVED, or RENAMED requirement)"
    fi
}

# validate_change - Validate a change directory
#
# Task 4.1: Implement validate_change() function with specs directory detection
# Task 4.2: Implement delta spec file discovery (find all spec.md under specs/)
#
# Validates all delta spec files in a change directory and the tasks file.
#
# Arguments:
#   $1 - change_dir: path to the change directory to validate
validate_change() {
    local change_dir="$1"
    local specs_dir="${change_dir}/specs"

    # Task 4.1: Check if specs directory exists
    if [[ ! -d "$specs_dir" ]]; then
        add_issue "ERROR" "$change_dir" 1 \
            "specs directory not found: ${specs_dir}"
        return
    fi

    # Task 4.2: Find all spec.md files under specs/
    # Use find command to recursively locate all spec.md files
    local spec_files=()
    while IFS= read -r -d '' spec_file; do
        spec_files+=("$spec_file")
    done < <(find "$specs_dir" -type f -name "spec.md" -print0 2>/dev/null | sort -z)

    if [[ ${#spec_files[@]} -eq 0 ]]; then
        add_issue "ERROR" "$specs_dir" 1 \
            "no spec.md files found in specs directory"
        return
    fi

    # Validate each delta spec file
    for spec_file in "${spec_files[@]}"; do
        validate_delta_spec_file "$spec_file"
    done

    # Validate tasks file if present (Tasks 5.1-5.5)
    validate_tasks_file "$change_dir"
}

# ============================================================================
# Section 5: Tasks File Validation (Tasks 5.1-5.5)
# ============================================================================

# validate_tasks_file - Validate a tasks.md file
#
# Task 5.1: Implement validate_tasks_file() function
#
# Validates that a tasks.md file (if present) contains at least one task item.
# Missing tasks.md is allowed - only an error if file exists but has no tasks.
#
# Arguments:
#   $1 - change_dir: path to the change directory
#
# Validation:
#   - Skip if tasks.md doesn't exist (not an error)
#   - Count lines matching TASK_ITEM_PATTERN
#   - Report ERROR if tasks.md exists but has zero tasks
validate_tasks_file() {
    local change_dir="$1"
    local tasks_path="${change_dir}/tasks.md"

    # Task 5.2: Check if tasks.md exists (skip if not present)
    # Missing tasks.md is allowed - not an error
    if [[ ! -f "$tasks_path" ]]; then
        return 0
    fi

    # Task 5.3: Count task items using flexible pattern
    # Count lines matching the task item pattern:
    #   - [ ] Task
    #   - [x] Completed task
    #   - [ ] 1.1 Numbered task
    #   - [X] Task (uppercase X)
    local task_count=0

    while IFS= read -r line || [[ -n "$line" ]]; do
        # Check if line matches the task item pattern
        if [[ "$line" =~ $TASK_ITEM_PATTERN ]]; then
            ((++task_count))
        fi
    done < "$tasks_path"

    # Task 5.4: Report error if tasks.md exists but has zero tasks
    if [[ "$task_count" -eq 0 ]]; then
        # Task 5.5: Provide helpful error message with expected format examples
        add_issue "ERROR" "$tasks_path" 1 \
            "tasks.md exists but contains no task items; expected format: '- [ ] Task' or '- [ ] N.N Task'"
    fi
}

# ============================================================================
# Section 6: Discovery Functions (Tasks 6.1-6.7)
# ============================================================================

# discover_specs - Discover all spec directories containing spec.md
#
# Task 6.1: Implement discover_specs() function using find command
# Task 6.2: Filter for directories under spectr/specs/ containing spec.md
# Task 6.3: Sort spec IDs alphabetically
# Task 6.7: Handle missing directories gracefully (return empty if directory doesn't exist)
#
# Returns:
#   Space-separated list of spec IDs (directory names), sorted alphabetically
#   Empty string if specs directory doesn't exist
#
# Matches behavior of: internal/discovery/specs.go:GetSpecs()
#
# Example output: "auth logging validation"
discover_specs() {
    # Task 6.7: Return empty if directory doesn't exist (not an error)
    if [[ ! -d "$SPECS_DIR" ]]; then
        return 0
    fi

    # Task 6.1, 6.2: Use find command to discover directories with spec.md
    # - mindepth 1, maxdepth 1: only direct subdirectories of specs/
    # - type d: directories only
    # - Check for spec.md file existence in each directory
    # - basename to get just the directory name (spec ID)
    # Task 6.3: Sort spec IDs alphabetically with sort command
    find "$SPECS_DIR" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | while read -r dir; do
        # Skip hidden directories (match Go implementation)
        local dirname
        dirname=$(basename "$dir")
        if [[ "$dirname" =~ ^\. ]]; then
            continue
        fi

        # Task 6.2: Filter for directories containing spec.md
        if [[ -f "$dir/spec.md" ]]; then
            echo "$dirname"
        fi
    done | sort
}

# discover_changes - Discover all active change directories
#
# Task 6.4: Implement discover_changes() function using find command
# Task 6.5: Filter for directories under spectr/changes/ excluding 'archive'
# Task 6.6: Sort change IDs alphabetically
# Task 6.7: Handle missing directories gracefully (return empty if directory doesn't exist)
#
# Returns:
#   Space-separated list of change IDs (directory names), sorted alphabetically
#   Empty string if changes directory doesn't exist
#
# Matches behavior of: internal/discovery/changes.go:GetActiveChanges()
#
# Example output: "add-feature fix-bug improve-performance"
discover_changes() {
    # Task 6.7: Return empty if directory doesn't exist (not an error)
    if [[ ! -d "$CHANGES_DIR" ]]; then
        return 0
    fi

    # Task 6.4, 6.5: Use find command to discover active change directories
    # - mindepth 1, maxdepth 1: only direct subdirectories of changes/
    # - type d: directories only
    # - Exclude 'archive' directory
    # - Check for proposal.md file existence (match Go implementation)
    # - basename to get just the directory name (change ID)
    # Task 6.6: Sort change IDs alphabetically with sort command
    find "$CHANGES_DIR" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | while read -r dir; do
        local dirname
        dirname=$(basename "$dir")

        # Skip hidden directories (match Go implementation)
        if [[ "$dirname" =~ ^\. ]]; then
            continue
        fi

        # Task 6.5: Exclude 'archive' directory
        if [[ "$dirname" == "archive" ]]; then
            continue
        fi

        # Check for proposal.md file existence (match Go implementation)
        if [[ -f "$dir/proposal.md" ]]; then
            echo "$dirname"
        fi
    done | sort
}

# ============================================================================
# Section 7: Validation Orchestration (Tasks 7.1-7.10)
# ============================================================================

# validate_single_spec - Validate a single spec by ID
#
# Task 7.1: Implement validate_single_spec() function
# Task 7.2: Verify spec directory exists before validation
# Task 7.3: Call validate_spec_file() for spec.md
#
# Validates a single spec directory and tracks it in the validated items list.
#
# Arguments:
#   $1 - spec_id: the spec ID (directory name under spectr/specs/)
#
# Adds to VALIDATED_ITEMS and ITEM_TYPES arrays for reporting
validate_single_spec() {
    local spec_id="$1"
    local spec_dir="$SPECS_DIR/$spec_id"
    local spec_file="$spec_dir/spec.md"

    # Track this item as being validated
    VALIDATED_ITEMS+=("$spec_id")
    ITEM_TYPES["$spec_id"]="spec"

    # Task 7.2: Verify spec directory exists before validation
    if [[ ! -d "$spec_dir" ]]; then
        add_issue "ERROR" "specs/$spec_id" 1 \
            "Spec directory not found: $spec_dir"
        return
    fi

    # Check if spec.md file exists
    if [[ ! -f "$spec_file" ]]; then
        add_issue "ERROR" "specs/$spec_id" 1 \
            "spec.md file not found in $spec_dir"
        return
    fi

    # Task 7.3: Call validate_spec_file() for spec.md
    validate_spec_file "$spec_file"
}

# validate_single_change - Validate a single change by ID
#
# Task 7.4: Implement validate_single_change() function
# Task 7.5: Verify change directory exists before validation
# Task 7.6: Call validate_change() for change directory
#
# Validates a single change directory and tracks it in the validated items list.
#
# Arguments:
#   $1 - change_id: the change ID (directory name under spectr/changes/)
#
# Adds to VALIDATED_ITEMS and ITEM_TYPES arrays for reporting
validate_single_change() {
    local change_id="$1"
    local change_dir="$CHANGES_DIR/$change_id"

    # Track this item as being validated
    VALIDATED_ITEMS+=("$change_id")
    ITEM_TYPES["$change_id"]="change"

    # Task 7.5: Verify change directory exists before validation
    if [[ ! -d "$change_dir" ]]; then
        add_issue "ERROR" "changes/$change_id" 1 \
            "Change directory not found: $change_dir"
        return
    fi

    # Task 7.6: Call validate_change() for change directory
    validate_change "$change_dir"
}

# validate_all - Validate all specs and changes
#
# Task 7.7: Implement validate_all() function
# Task 7.8: Discover and validate all specs
# Task 7.9: Discover and validate all changes
# Task 7.10: Aggregate results across all items
#
# Discovers and validates all specs and changes in the repository.
# Results are aggregated in the ISSUES, VALIDATED_ITEMS, and ITEM_TYPES arrays.
validate_all() {
    # Task 7.8: Discover and validate all specs
    local specs
    specs=$(discover_specs)

    # Validate each discovered spec
    if [[ -n "$specs" ]]; then
        while IFS= read -r spec_id; do
            validate_single_spec "$spec_id"
        done <<< "$specs"
    fi

    # Task 7.9: Discover and validate all changes
    local changes
    changes=$(discover_changes)

    # Validate each discovered change
    if [[ -n "$changes" ]]; then
        while IFS= read -r change_id; do
            validate_single_change "$change_id"
        done <<< "$changes"
    fi

    # Task 7.10: Aggregate results across all items
    # Results are already aggregated in:
    # - ISSUES array: all validation issues
    # - ISSUE_COUNTS: error and warning counts
    # - VALIDATED_ITEMS: list of validated item names
    # - ITEM_TYPES: type (spec/change) for each validated item
    # These will be used by output formatting functions
}

# ============================================================================
# Section 8: Output Formatting (Tasks 8.1-8.12)
# ============================================================================

# make_relative_path - Convert absolute path to relative path from spectr/
#
# Converts an absolute path to a path relative to the spectr/ directory.
# Removes everything up to and including the spectr/ prefix.
#
# This ensures output paths match the format from the Go binary:
#   "changes/foo/spec.md" instead of "spectr/changes/foo/spec.md"
#   "specs/bar/spec.md" instead of "/abs/path/spectr/specs/bar/spec.md"
#
# Arguments:
#   $1 - abs_path: absolute path to convert
#
# Returns:
#   Relative path from spectr/ directory
#
# Example:
#   /home/user/project/spectr/changes/foo/spec.md -> changes/foo/spec.md
#   spectr/specs/bar/spec.md -> specs/bar/spec.md
make_relative_path() {
    local abs_path="$1"
    local spectr_dir="${SPECTR_DIR}"

    # Remove spectr/ prefix if present
    # Uses parameter expansion ${var#*pattern} to remove shortest match from start
    if [[ "$abs_path" == *"${spectr_dir}/"* ]]; then
        echo "${abs_path#*${spectr_dir}/}"
    else
        # If no spectr/ found, return path as-is (shouldn't happen in normal use)
        echo "$abs_path"
    fi
}

# format_level - Format error level with color codes
#
# Formats [ERROR] in red, [WARNING] in yellow, with colors only if TTY.
#
# Arguments:
#   $1 - level: ERROR or WARNING
#
# Returns:
#   Formatted level string with color codes (if TTY)
format_level() {
    local level="$1"

    case "$level" in
        ERROR)
            echo -e "${RED}[ERROR]${NC}"
            ;;
        WARNING)
            echo -e "${YELLOW}[WARNING]${NC}"
            ;;
        *)
            echo "[$level]"
            ;;
    esac
}

# print_human_results - Print validation results in human-readable format
#
# Task 8.1: Implement print_human_results() function
# Task 8.2: Group issues by file path for cleaner output
# Task 8.3: Print file headers followed by indented issues
# Task 8.4: Apply color codes to error levels ([ERROR] in red, [WARNING] in yellow)
# Task 8.5: Format issue lines with "line X: message" pattern
# Task 8.6: Implement summary line (X passed, Y failed (E errors), Z total)
# Task 8.7: Add blank line separators between failed items
#
# Prints validation results in human-readable format matching the Go
# implementation's output (internal/validation/formatters.go:PrintBulkHumanResults)
#
# Output format:
#   changes/add-feature/specs/auth/spec.md
#     [ERROR] line 15: ADDED requirement must have at least one scenario
#     [ERROR] line 23: ADDED requirement must contain SHALL or MUST
#
#   specs/validation/spec.md
#     [ERROR] line 1: Missing required '## Requirements' section
#
#   Summary: 2 passed, 2 failed (3 errors), 4 total
print_human_results() {
    local total_items=${#VALIDATED_ITEMS[@]}
    local passed_items=0
    local failed_items=0
    local total_errors=${ISSUE_COUNTS[errors]}
    local total_warnings=${ISSUE_COUNTS[warnings]}

    # Task 8.2: Group issues by file path
    # Use associative array to group issues by file
    declare -A issues_by_file
    declare -a file_order=()  # Track order of first occurrence

    # Group all issues by their file path
    if [[ ${#ISSUES[@]} -gt 0 ]]; then
        for issue in "${ISSUES[@]}"; do
            IFS='|' read -r level path line message <<< "$issue"
            local rel_path
            rel_path=$(make_relative_path "$path")

            # Track file order on first occurrence
            if [[ ! -v "issues_by_file[$rel_path]" ]]; then
                file_order+=("$rel_path")
                issues_by_file[$rel_path]=""
            fi

            # Append issue to file's issue list (with newline separator)
            issues_by_file[$rel_path]+="$(format_level "$level") line ${line}: ${message}
"
        done
    fi

    # Track which items have issues (failed items)
    declare -A failed_item_lookup

    # Only process if there are files with issues
    if [[ ${#file_order[@]} -gt 0 ]]; then
        for file in "${file_order[@]}"; do
            # Determine which item this file belongs to
            local item_name=""
            for item in "${VALIDATED_ITEMS[@]}"; do
                local item_type="${ITEM_TYPES[$item]}"
                if [[ "$item_type" == "spec" ]] && [[ "$file" == "specs/${item}/"* ]]; then
                    item_name="$item"
                    break
                elif [[ "$item_type" == "change" ]] && [[ "$file" == "changes/${item}/"* ]]; then
                    item_name="$item"
                    break
                fi
            done

            if [[ -n "$item_name" ]]; then
                failed_item_lookup[$item_name]=1
            fi
        done
    fi

    # Count passed/failed items
    if [[ ${#VALIDATED_ITEMS[@]} -gt 0 ]]; then
        for item in "${VALIDATED_ITEMS[@]}"; do
            # Use ${failed_item_lookup[$item]:-} to handle unset keys with set -u
            if [[ -n "${failed_item_lookup[$item]:-}" ]]; then
                ((++failed_items))
            else
                ((++passed_items))
            fi
        done
    fi

    # Task 8.3: Print file headers followed by indented issues
    # Task 8.7: Add blank line separators between failed items
    local is_first=true

    # Only print file issues if there are any files with issues
    if [[ ${#file_order[@]} -gt 0 ]]; then
        for file in "${file_order[@]}"; do
            # Add blank line before each file (except first)
            if ! $is_first; then
                echo
            fi
            is_first=false

            # Print file path as header
            echo "$file"

            # Task 8.3: Print indented issues
            # Task 8.5: Format issue lines with "line X: message" pattern
            # Issues are already formatted with "line X: message" from grouping step
            echo -n "${issues_by_file[$file]}" | sed 's/^/  /'
        done
    fi

    # Task 8.6: Implement summary line (X passed, Y failed (E errors), Z total)
    if [[ $failed_items -gt 0 ]]; then
        echo
        echo "Summary: $passed_items passed, $failed_items failed ($total_errors errors), $total_items total"
    else
        echo
        echo "Summary: $total_items passed, 0 failed, $total_items total"
    fi
}

# check_json_capability - Check if jq is available
#
# Tests whether the jq JSON processor is installed and available in PATH.
# This is used to determine whether we can generate structured JSON output
# or need to fall back to human-readable output.
#
# Returns:
#   0 if jq is available (command found in PATH)
#   1 if jq is not available (command not found)
#
# Note: The command output is redirected to /dev/null to suppress "not found" messages
check_json_capability() {
    if command -v jq >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# json_escape - Escape string for JSON
#
# Escapes special JSON characters (backslash, quote, newline, etc.)
# to prevent JSON syntax errors and injection attacks.
#
# This function is used in the fallback printf-based JSON generation
# when jq is not available. When jq is available, it handles escaping
# automatically.
#
# Arguments:
#   $1 - str: string to escape
#
# Returns:
#   Escaped string suitable for JSON values
#
# Example:
#   json_escape 'He said "hello"' -> He said \"hello\"
#   json_escape 'Line 1\nLine 2' -> Line 1\\nLine 2
json_escape() {
    local str="$1"
    # Escape special JSON characters using bash parameter expansion
    # Order matters: backslash must be escaped first to avoid double-escaping
    str="${str//\\/\\\\}"    # Backslash: \ -> \\
    str="${str//\"/\\\"}"    # Quote: " -> \"
    str="${str//$'\n'/\\n}"  # Newline: \n -> \\n
    str="${str//$'\r'/\\r}"  # Carriage return: \r -> \\r
    str="${str//$'\t'/\\t}"  # Tab: \t -> \\t
    echo "$str"
}

# issue_belongs_to_item - Check if issue belongs to item
#
# Determines if an issue path belongs to a specific item.
#
# Arguments:
#   $1 - item_name: name of the item
#   $2 - item_type: "spec" or "change"
#   $3 - issue_path: path from the issue
#
# Returns:
#   0 (true) if issue belongs to item, 1 (false) otherwise
issue_belongs_to_item() {
    local item_name="$1"
    local item_type="$2"
    local issue_path="$3"

    if [[ "$item_type" == "spec" ]]; then
        # Check if path starts with specs/{item_name}/
        if [[ "$issue_path" == *"specs/${item_name}/"* ]] || \
           [[ "$issue_path" == "specs/${item_name}" ]]; then
            return 0
        fi
    elif [[ "$item_type" == "change" ]]; then
        # Check if path starts with changes/{item_name}/
        if [[ "$issue_path" == *"changes/${item_name}/"* ]] || \
           [[ "$issue_path" == "changes/${item_name}" ]]; then
            return 0
        fi
    fi

    return 1
}

# print_json_results - Print validation results in JSON format
#
# Task 8.8: Implement print_json_results() function
# Task 8.9: Generate JSON structure with version, items, summary fields
# Task 8.10: Use jq for JSON generation if available
# Task 8.11: Fallback to printf-based JSON if jq unavailable
# Task 8.12: Warn user if --json requested but jq not found
#
# Prints validation results in JSON format matching the Go implementation's
# structure (internal/validation/formatters.go:PrintBulkJSONResults)
#
# JSON structure:
#   {
#     "version": 1,
#     "items": [
#       {
#         "name": "add-feature",
#         "type": "change",
#         "valid": false,
#         "issues": [
#           {
#             "level": "ERROR",
#             "path": "changes/add-feature/specs/auth/spec.md",
#             "line": 15,
#             "message": "ADDED requirement must have at least one scenario"
#           }
#         ]
#       }
#     ],
#     "summary": {
#       "total": 2,
#       "passed": 1,
#       "failed": 1,
#       "errors": 1,
#       "warnings": 0
#     }
#   }
print_json_results() {
    # Task 8.10: Use jq for JSON generation if available
    if check_json_capability; then
        print_json_with_jq
    else
        # Task 8.11: Fallback to printf-based JSON if jq unavailable
        # Task 8.12: Warn user if --json requested but jq not found
        echo "Warning: jq not found, falling back to printf-based JSON generation" >&2
        print_json_with_printf
    fi
}

# print_json_with_jq - Generate JSON using jq
#
# Task 8.10: Use jq for JSON generation if available
# Task 8.9: Generate JSON structure with version, items, summary fields
print_json_with_jq() {
    local total_items=${#VALIDATED_ITEMS[@]}
    local passed_items=0
    local failed_items=0

    # Build items array JSON
    local items_json="[]"

    for item_name in "${VALIDATED_ITEMS[@]}"; do
        local item_type="${ITEM_TYPES[$item_name]}"
        local item_issues_json="[]"
        local is_valid="true"

        # Find issues for this item
        if [[ ${#ISSUES[@]} -gt 0 ]]; then
            for issue in "${ISSUES[@]}"; do
                IFS='|' read -r level path line message <<< "$issue"

                # Check if issue belongs to this item
                if issue_belongs_to_item "$item_name" "$item_type" "$path"; then
                    is_valid="false"
                    local rel_path
                    rel_path=$(make_relative_path "$path")

                    # Add issue to item's issues array using jq
                    item_issues_json=$(echo "$item_issues_json" | jq \
                        --arg level "$level" \
                        --arg path "$rel_path" \
                        --arg line "$line" \
                        --arg msg "$message" \
                        '. + [{level: $level, path: $path, line: ($line | tonumber), message: $msg}]')
                fi
            done
        fi

        # Count passed/failed
        if [[ "$is_valid" == "true" ]]; then
            ((++passed_items))
        else
            ((++failed_items))
        fi

        # Add item to items array
        items_json=$(echo "$items_json" | jq \
            --arg name "$item_name" \
            --arg type "$item_type" \
            --argjson valid "$is_valid" \
            --argjson issues "$item_issues_json" \
            '. + [{name: $name, type: $type, valid: $valid, issues: $issues}]')
    done

    # Build final JSON with jq
    jq -n \
        --argjson items "$items_json" \
        --argjson total "$total_items" \
        --argjson passed "$passed_items" \
        --argjson failed "$failed_items" \
        --argjson errors "${ISSUE_COUNTS[errors]}" \
        --argjson warnings "${ISSUE_COUNTS[warnings]}" \
        '{
            version: 1,
            items: $items,
            summary: {
                total: $total,
                passed: $passed,
                failed: $failed,
                errors: $errors,
                warnings: $warnings
            }
        }'
}

# print_json_with_printf - Generate JSON using printf (fallback)
#
# Task 8.11: Fallback to printf-based JSON if jq unavailable
# Task 8.9: Generate JSON structure with version, items, summary fields
print_json_with_printf() {
    local total_items=${#VALIDATED_ITEMS[@]}
    local passed_items=0
    local failed_items=0

    # Start JSON object
    echo "{"
    echo "  \"version\": 1,"
    echo "  \"items\": ["

    local first_item=true
    for item_name in "${VALIDATED_ITEMS[@]}"; do
        local item_type="${ITEM_TYPES[$item_name]}"
        local is_valid=true
        local item_issues=()

        # Find issues for this item
        if [[ ${#ISSUES[@]} -gt 0 ]]; then
            for issue in "${ISSUES[@]}"; do
                IFS='|' read -r level path line message <<< "$issue"

                # Check if issue belongs to this item
                if issue_belongs_to_item "$item_name" "$item_type" "$path"; then
                    is_valid=false
                    local rel_path
                    rel_path=$(make_relative_path "$path")
                    item_issues+=("$level|$rel_path|$line|$message")
                fi
            done
        fi

        # Count passed/failed
        if $is_valid; then
            ((++passed_items))
        else
            ((++failed_items))
        fi

        # Print item separator (comma before all but first item)
        if ! $first_item; then
            echo ","
        fi
        first_item=false

        # Print item object
        echo "    {"
        echo "      \"name\": \"$(json_escape "$item_name")\","
        echo "      \"type\": \"$item_type\","
        if $is_valid; then
            echo "      \"valid\": true,"
        else
            echo "      \"valid\": false,"
        fi
        echo "      \"issues\": ["

        # Print issues
        local first_issue=true
        if [[ ${#item_issues[@]} -gt 0 ]]; then
            for item_issue in "${item_issues[@]}"; do
                IFS='|' read -r level path line message <<< "$item_issue"

                if ! $first_issue; then
                    echo ","
                fi
                first_issue=false

                echo "        {"
                echo "          \"level\": \"$level\","
                echo "          \"path\": \"$(json_escape "$path")\","
                echo "          \"line\": $line,"
                echo "          \"message\": \"$(json_escape "$message")\""
                echo -n "        }"
            done
        fi

        echo
        echo "      ]"
        echo -n "    }"
    done

    echo
    echo "  ],"
    echo "  \"summary\": {"
    echo "    \"total\": $total_items,"
    echo "    \"passed\": $passed_items,"
    echo "    \"failed\": $failed_items,"
    echo "    \"errors\": ${ISSUE_COUNTS[errors]},"
    echo "    \"warnings\": ${ISSUE_COUNTS[warnings]}"
    echo "  }"
    echo "}"
}

# ============================================================================
# Section 9: Argument Parsing and Main Entry Point (Tasks 9.1-9.11)
# ============================================================================

# print_usage - Print usage message
#
# Task 9.1: Implement print_usage() function with flag documentation
#
# Prints comprehensive usage information including all flags, examples,
# exit codes, and environment variables.
print_usage() {
    cat << EOF
Usage: validate.sh [OPTIONS]

Validate Spectr specifications and change proposals.

Options:
  --spec <spec-id>       Validate a single spec by ID
  --change <change-id>   Validate a single change by ID
  --all                  Validate all specs and changes
  --json                 Output JSON instead of human-readable format
  -h, --help             Show this help message

Examples:
  # Validate a single spec
  validate.sh --spec validation

  # Validate a single change
  validate.sh --change add-feature

  # Validate everything
  validate.sh --all

  # JSON output for programmatic use
  validate.sh --all --json

Environment Variables:
  SPECTR_DIR             Custom spectr directory location (default: "spectr")

Exit Codes:
  0                      All validations passed (no errors)
  1                      One or more validations failed (errors found)
  2                      Usage error (invalid arguments)

Notes:
  - Exactly one mode flag (--spec, --change, or --all) is required
  - The --json flag can be combined with any mode
  - JSON output requires jq (falls back to human-readable if unavailable)
EOF
}

# main - Main entry point with argument parsing and validation orchestration
#
# Task 9.2: Implement main() function with argument parsing loop
# Task 9.3: Parse --spec <id> flag and store mode/target
# Task 9.4: Parse --change <id> flag and store mode/target
# Task 9.5: Parse --all flag and set mode
# Task 9.6: Parse --json flag and set json_output boolean
# Task 9.7: Parse -h/--help flag and print usage
# Task 9.8: Validate required arguments (error if mode not specified)
# Task 9.9: Execute validation based on mode (spec/change/all)
# Task 9.10: Output results based on json_output flag
# Task 9.11: Exit with code 0 if no errors, 1 if errors, 2 if usage error
main() {
    local mode=""
    local target=""
    local json_output=false

    # Task 9.2: Argument parsing loop
    while [[ $# -gt 0 ]]; do
        case "$1" in
            # Task 9.3: Parse --spec <id> flag and store mode/target
            --spec)
                if [[ -z "$2" ]]; then
                    echo "Error: --spec requires an argument" >&2
                    print_usage
                    exit 2  # Task 9.11: Exit with code 2 for usage error
                fi
                if [[ -n "$mode" ]]; then
                    echo "Error: Cannot specify multiple modes (--spec, --change, --all)" >&2
                    print_usage
                    exit 2
                fi
                mode="spec"
                target="$2"
                shift 2
                ;;

            # Task 9.4: Parse --change <id> flag and store mode/target
            --change)
                if [[ -z "$2" ]]; then
                    echo "Error: --change requires an argument" >&2
                    print_usage
                    exit 2
                fi
                if [[ -n "$mode" ]]; then
                    echo "Error: Cannot specify multiple modes (--spec, --change, --all)" >&2
                    print_usage
                    exit 2
                fi
                mode="change"
                target="$2"
                shift 2
                ;;

            # Task 9.5: Parse --all flag and set mode
            --all)
                if [[ -n "$mode" ]]; then
                    echo "Error: Cannot specify multiple modes (--spec, --change, --all)" >&2
                    print_usage
                    exit 2
                fi
                mode="all"
                shift
                ;;

            # Task 9.6: Parse --json flag and set json_output boolean
            --json)
                json_output=true
                shift
                ;;

            # Task 9.7: Parse -h/--help flag and print usage
            -h|--help)
                print_usage
                exit 0
                ;;

            # Unknown flag - usage error
            *)
                echo "Error: Unknown option: $1" >&2
                print_usage
                exit 2
                ;;
        esac
    done

    # Task 9.8: Validate required arguments (error if mode not specified)
    if [[ -z "$mode" ]]; then
        echo "Error: Must specify --spec, --change, or --all" >&2
        print_usage
        exit 2
    fi

    # Task 9.9: Execute validation based on mode (spec/change/all)
    case "$mode" in
        spec)
            validate_single_spec "$target"
            ;;
        change)
            validate_single_change "$target"
            ;;
        all)
            validate_all
            ;;
    esac

    # Task 9.10: Output results based on json_output flag
    if $json_output; then
        # Check if jq is available for JSON output
        if check_json_capability; then
            print_json_results
        else
            # Warn and fallback to human-readable output
            echo "Warning: jq not found, falling back to human-readable output" >&2
            print_human_results
        fi
    else
        print_human_results
    fi

    # Task 9.11: Exit with code 0 if no errors, 1 if errors, 2 if usage error
    # (Usage errors handled above with exit 2)
    if [[ ${ISSUE_COUNTS[errors]} -gt 0 ]]; then
        exit 1  # Exit with code 1 if validation errors found
    fi
    exit 0  # Exit with code 0 if all validations passed
}

# Script execution
main "$@"

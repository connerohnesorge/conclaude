# Proposal Creation Guide

## Guardrails

- Favor straightforward, minimal implementations first and add complexity
  only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `spectr/AGENTS.md` and `spectr/project.md` (located inside the
  `spectr/` directory—run `ls spectr`) if you need additional
  Spectr conventions or clarifications.
- Identify any vague or ambiguous details and ask the necessary follow-up
  questions before editing files.

Note: You are not implementing yet, you are fully planning and creating the change proposal using spectr.

## Steps

1. Review `spectr/project.md`, read `spectr/specs/` and
   `spectr/changes/` directories, and inspect related code or docs (e.g.,
   via `rg`/`ls`) to ground the proposal in current behaviour; note any gaps
   that require clarification.
2. Choose a unique verb-led `change-id` and scaffold `proposal.md`,
   `tasks.md`, and `design.md` (when needed) under
   `spectr/changes/<id>/`.
3. Map the change into concrete capabilities or requirements, breaking
   multi-scope efforts into distinct spec deltas with clear relationships and
   sequencing.
4. Capture architectural reasoning in `design.md` when the solution spans
   multiple systems, introduces new patterns, or demands trade-off discussion
   before committing to specs.
5. Draft spec deltas in `spectr/changes/<id>/specs/<capability>/spec.md`
   (one folder per capability) using `## ADDED|MODIFIED|REMOVED Requirements`
   with at least one `#### Scenario:` per requirement and cross-reference
   related capabilities when relevant.
6. Draft `tasks.md` as an ordered list of small, verifiable work items that
   deliver user-visible progress, include validation (tests, tooling), and
   highlight dependencies or parallelizable work. Note: After running `spectr
   accept`, both `tasks.md` (human-readable) and `tasks.jsonc`
   (machine-readable) will coexist—the former preserves formatting and context,
   while the latter becomes the runtime source of truth.
7. Validate with `spectr validate <id>` and resolve every issue before
   sharing the proposal.

## Reference

- Read delta specs directly at
  `spectr/changes/<id>/specs/<capability>/spec.md` when validation fails.
- Read existing specs at `spectr/specs/<capability>/spec.md` to understand
  current state.
- Search existing requirements with `rg -n "Requirement:|Scenario:"
  spectr/specs` before writing new ones.
- Explore the codebase with `rg <keyword>`, `ls`, or direct file reads so
  proposals align with current implementation realities.

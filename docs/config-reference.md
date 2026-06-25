# `.loopkit.yaml` Reference

loopkit is fully config-driven. Every field has a sensible default, but all vocabulary and enforced states come from this file. No state names, halt reasons, or verbs are hardcoded in the binary.

## Complete schema

```yaml
# Where skills live relative to project root (default: "skills/")
skills_dir: skills/

# Max iterations for budget-constrained simulation (default: 20)
max_iterations: 20

# Declared verb vocabulary — flags non-standard verbs in LOOP.md steps
standard_verbs:
  - trigger
  - handoff
  - halt
  - call
  - wait
  - route
  - escalate
  - resume
  - notify
  - complete

# Declared halt reasons — flags unknown reasons in transition rules
halt_reasons:
  - stall
  - ambiguous
  - human-gate
  - unsafe
  - budget

# Required LOOP.md sections in canonical order (default: 7 below)
canonical_loop_sections:
  - Entry Conditions
  - Loop State Schema
  - Single Iteration Step
  - Proof of Progress
  - State Transition Rule
  - Halt Conditions
  - Handoff Target

# Required SKILL.md sections (default: 3 below)
canonical_skill_sections:
  - Description
  - Rules
  - State Model

# Alternative heading names for the state model section
state_model_aliases:
  - State Model
  - The Loop
  - Loop States
  - States

# States that MUST appear in at least one transition
enforced_states:
  - name: in-dev
    agent: developer-agent
    description: Developer builds story AC by AC
  - name: done
    agent: ""
    description: Story deployed and verified
  - name: halted-stall
    agent: ""
    description: No progress for N iterations

# Opt-in: desk check pattern validation (all fields required when enabled)
deskcheck_enabled: true
deskcheck_state: in-deskcheck          # the desk check state name
deskcheck_entry_from: in-dev           # which state enters desk check
deskcheck_feedback_to: in-dev          # bug feedback return path
deskcheck_forward_to: in-qa            # completion forward path

# Opt-in: bug feedback loop validation (all fields required when enabled)
bug_feedback_enabled: true
bug_feedback_qa_state: in-qa           # QA state that must feed back
bug_feedback_acceptance_state: in-acceptance  # acceptance state that must feed back
bug_feedback_return_to: in-dev         # where bugs return to
```

## Field reference

### `skills_dir`
Path to the skills directory, relative to the project root. loopkit scans this for skill directories containing `SKILL.md`.

**Default:** `skills/`

**Discovery order:** flat (depth 1) first, then nested (depth 2) as fallback for legacy projects.

### `max_iterations`
Budget ceiling for the simulation validator. Ensures every entry state can reach a terminal state within this iteration count.

**Default:** `20`

### `standard_verbs`
The declared vocabulary of action verbs. Any verb in a LOOP.md numbered step that isn't in this list triggers a `loop-nonstandard-verb` warning. Common temporal conjunctions (if, when, after) and structural words (the, a, this) are automatically skipped.

**Default:** `[trigger, handoff, halt, call, wait, route, escalate, resume, notify, complete]`

### `halt_reasons`
Declared halt reasons. Any `halt <reason>` in a transition rule with a reason not in this list triggers a `loop-unknown-halt` warning. The word following `halt` is compared against this list after skipping noise words like "the", "when", "if".

**Default:** `[stall, ambiguous, human-gate, unsafe, budget]`

### `canonical_loop_sections`
The required sections for LOOP.md, in their canonical order. loopkit checks:
- All required sections are present
- Sections appear in the specified order
- No unknown top-level H2 headings exist

**Default:** 7 sections (Entry Conditions through Handoff Target)

### `canonical_skill_sections`
Required sections for SKILL.md. Checks presence only (not order).

**Default:** `[Description, Rules, State Model]`

### `state_model_aliases`
Alternative heading names that count as the "State Model" section. Useful for projects that use different terminology.

**Default:** `[State Model, The Loop, Loop States, States]`

### `enforced_states`
States that must appear somewhere in the combined transition graph of all skills. Each entry defines:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | The state name (kebab-case, lowercase) |
| `agent` | No | The agent responsible for this state |
| `description` | No | Human-readable description of the state |

If a state in this list doesn't appear in any transition (as either source or target), loopkit emits a `state-enforced-missing` error.

**Default:** `[]` (empty — no hardcoded states)

### `deskcheck_*`
Opt-in validation of the desk check sub-pattern. When `deskcheck_enabled` is `true`, loopkit checks:

1. `deskcheck_entry_from` → `deskcheck_state` (entry edge exists)
2. `deskcheck_state` → `deskcheck_feedback_to` (bug feedback edge exists)
3. `deskcheck_state` → `deskcheck_forward_to` (completion edge exists)

Only fires if `deskcheck_state` appears in the graph. All string fields must be non-empty when enabled.

**Default:** disabled (`deskcheck_enabled: false`)

### `bug_feedback_*`
Opt-in validation of bug feedback loops. When `bug_feedback_enabled` is `true`, loopkit checks:

1. `bug_feedback_qa_state` → `bug_feedback_return_to` (QA found bugs)
2. `bug_feedback_acceptance_state` → `bug_feedback_return_to` (PO/UX found issues)

Only fires for states that actually appear in the graph. All string fields must be non-empty when enabled.

**Default:** disabled (`bug_feedback_enabled: false`)

## Why no defaults for enforced states?

loopkit ships with **zero hardcoded state names**. This is by design:

- Your team's workflow uses `in-progress` not `in-dev`? loopkit doesn't care — configure it.
- Your terminal state is called `shipped` not `done`? Configure it.
- You don't use a desk check step at all? Don't enable the validator.
- Your bug feedback loops go to `triage` not back to `in-dev`? Configure it.

Every project defines its own vocabulary. loopkit just enforces that the vocabulary is used consistently.

## Example: minimal config

For a simple project with two states:

```yaml
skills_dir: skills/
enforced_states:
  - name: in-progress
    agent: developer
  - name: done
    agent: ""
```

## Example: full forge config

See the [forge `.loopkit.yaml`](https://github.com/loopworx/forge/blob/main/.loopkit.yaml) for a production example with 10 enforced states, 40 standard verbs, desk check and bug feedback validators enabled.

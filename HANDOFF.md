# loopkit — Session Handoff

**Date:** 2026-06-24
**Repo:** github.com/loopworx/loopkit
**Local path:** /Users/canavar/projects/skill-loop-verifier

## What Is loopkit?

A CLI tool that validates any directory of Claude agent skills. It proves:
1. **Loop soundness** — state machine is complete, consistent, free of dead-ends
2. **Best-practices compliance** — obeys Claude Skill Best Practices guide

Single binary, one run, nothing optional.

## Architecture

Three-crate Cargo workspace:

```
loopkit/
├── Cargo.toml              (workspace root)
├── crates/
│   ├── loopkit-core/       (types, parsers, discovery — no validation)
│   ├── loopkit-graph/      (graph, simulation, state/transition checks)
│   └── loopkit/            (CLI binary + best-practices checks)
```

Dependency flow: `loopkit-core` ← `loopkit-graph` ← `loopkit` (CLI)

## Key Documents

- **Design spec:** `docs/superpowers/specs/2026-06-24-loopkit-design.md`
- **Implementation plan:** `docs/superpowers/plans/2026-06-24-loopkit-implementation.md`

Read both before continuing. The spec has the full design; the plan has 28 tasks with step-by-step instructions.

## Progress

### Completed (Phase 0 + Phase 1 + most of Phase 2)

| Task | Status | Commit |
|---|---|---|
| 0.1: Workspace scaffold | ✅ Done | `ad9057f` |
| 1.1: Port universal types to loopkit-core | ✅ Done | `9456a51` |
| 1.2: Port SKILL.md parser (gap fixes R4-C2, R4-C4) | ✅ Done | `7d94b32` |
| 1.3: Port YAML config parser | ✅ Done | `473400a` |
| 1.4: Port config loader | ✅ Done | `473400a` |
| 1.5: Expand Config with enforced language | ✅ Done | `8621b4a` |
| 1.6: State-name validation utility | ✅ Done | `71713f0` |
| 2.1: Port loop-specific types | ✅ Done | `a204d8c` |
| 2.2: Port loop parsers (gap fixes R4-C1, R4-C3, R4 halt, R4 dup) | ✅ Done | (in 2.4 commit) |
| 2.3: Port graph builder | ✅ Done | `0f14bea` |
| 2.4: Port existing validators with gap fixes | ✅ Done | (large commit) |
| 2.5: Enforced states validator (NEW) | ✅ Done | `a4f1b1e` |
| 2.6: Deskcheck pattern validator (NEW) | ✅ Done | `39b953e` |
| 2.7: Bug feedback validator (NEW) | ✅ Done | `02eeefe` |

### Not Started

| Task | Description |
|---|---|
| 2.8 | Port tests to loopkit-graph (copy old tests, update imports) |
| 3.1-3.8 | Best-practices validators (frontmatter, naming, structure, progressive, terminology, workflow, antipatterns, orchestrator) |
| 4.1 | Wire CLI main with clap |
| 4.2 | Run against forge, fix issues |
| 5.1 | Update .loopkit.yaml |
| 5.2 | Build release binary |

## Next Steps (in order)

1. **Task 2.8:** Port existing tests from old `tests/` directory to `crates/loopkit-graph/tests/`. Update imports from `skill_loop_verifier::*` to `loopkit_graph::*` and `loopkit_core::types::*`.

2. **Task 3.1-3.8:** Create best-practices validators in `crates/loopkit/src/best_practices/`. See plan for exact code. Categories:
   - frontmatter.rs (8 checks: name/description validation)
   - naming.rs (3 checks: gerund, vague, consistency)
   - structure.rs (5 checks: line count, deep refs, TOC, Windows paths, time-sensitive)
   - progressive.rs (2 checks: no progressive disclosure, orphan refs)
   - terminology.rs (1 check: synonym detection)
   - workflow.rs (3 checks: checklist, feedback loop, conditionals)
   - antipatterns.rs (3 checks: too many options, magic numbers, punting)
   - mod.rs (orchestrator: check_all function)

3. **Task 4.1:** Wire CLI in `crates/loopkit/src/main.rs` using clap. Calls `loopkit_graph::validators::run_all()` + `best_practices::check_all()`, merges diagnostics, formats output.

4. **Task 4.2:** Run `cargo run -p loopkit -- /Users/canavar/projects/forge` and fix any errors.

5. **Task 5.1-5.2:** Create `.loopkit.yaml` config file, build release binary.

## Enforced State Model

Default enforced states (in `crates/loopkit-core/src/types.rs`):

```
backlog → in-analysis → in-dev → in-deskcheck → in-qa → in-acceptance → ready-for-deploy → done
  ↑ coordinator    ↑ po      ↑ dev     ↑ qa (per AC)   ↑ qa   ↑ po + ux        ↑ human

Bug feedback:
  in-deskcheck → in-dev    (QA finds issue per AC)
  in-qa → in-dev           (QA finds issue, full check)
  in-acceptance → in-dev   (PO/UX find issue)

Halt states:
  halted-stall, halted-human-gate, halted-unsafe
```

## Gap Fixes Applied

| Gap | Fix |
|---|---|
| ASCII `->` not recognized | Parser accepts `→`, `->`, `--->` |
| Section body extraction mismatch | Uses pulldown_cmark event stream, not `str::find` |
| `## State Model` hardcoded | Uses `config.state_model_aliases` |
| No enforced-state validation | New `enforced_states.rs` validator |
| Transition endpoints not validated | Reverse check in `state_consistency.rs` |
| Dead `TransitionToUnknownState` | Removed from simulation |
| Compound verb false positives | "Hand off" → "handoff", skip "After"/"Before" |
| `has_all_canonical_sections` duplicate bug | Uses HashSet for unique count |
| `halt after XYZ` → `Some(0)` | Uses `and_then(parse::ok)` |
| Hardcoded `20` for max_iterations | Uses `config.max_iterations` |
| Missing name silently drops skill | Returns `Err(Diagnostic)` instead of `None` |
| Backtick-quoted states dropped | Strips backticks before regex match |

## Config Structure

`.loopkit.yaml` (optional, overrides defaults):

```yaml
skills_dir: skills/
max_iterations: 20
standard_verbs: [trigger, handoff, halt, call, wait, route, escalate, resume, notify, complete]
halt_reasons: [stall, ambiguous, human-gate, unsafe, budget]
canonical_loop_sections: [Entry Conditions, Loop State Schema, ...]
canonical_skill_sections: [Description, Rules, State Model, ...]
state_model_aliases: [State Model, The Loop, Loop States, States]
enforced_states:
  - name: backlog
    agent: coordinator
    description: Picks stories from Linear/Jira/Trello
  # ... (see types.rs for full default list)
```

## Build

```bash
cargo build          # build all crates
cargo test           # run all tests
cargo run -p loopkit -- /path/to/skills/   # run the tool
cargo run -p loopkit -- /path/to/skills/ --json  # JSON output
```

Exit codes: 0 = clean, 1 = errors found, 2 = internal error

## Important Notes

- **Cargo is not available in the OpenWork shell environment.** Tests haven't been run since the workspace restructure. The first thing to do in OpenCode is `cargo build` and fix any compilation errors.
- The old flat-crate files (`src/`, `tests/`) were deleted. Tests need to be ported (Task 2.8).
- The `loopkit` CLI crate (`crates/loopkit/src/main.rs`) currently just prints "loopkit v0.3.0" — it needs the real CLI wiring (Task 4.1).
- Best-practices validators don't exist yet (Tasks 3.1-3.8).
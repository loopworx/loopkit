# Skill Typechecker — Design Spec

**Date:** 2026-06-24
**Status:** Draft, pending review
**Replaces:** skill-loop-verifier v0.2.0

## 1. Purpose

A single binary that validates any directory of Claude agent skills. For every skill it proves:

1. **Loop soundness** — the state machine is complete, consistent, and free of dead-ends. All transitions use declared vocabulary. Required states and exits are present.
2. **Best-practices compliance** — the skill obeys the rules in the [Claude Skill Best Practices guide](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices).

Nothing is optional. One run checks everything.

---

## 2. Architecture

Three crates in a Cargo workspace, layered by responsibility:

```
skill-typechecker/
├── Cargo.toml
└── crates/
    ├── skill-core/        # Types, parsers, discovery — no validation
    ├── skill-loop/        # Graph, simulation, state/transition checks
    └── skill-typecheck/   # CLI binary + best-practices checks
```

### 2.1 `skill-core` — universal, shared

**Owns:** `Skill`, `Section`, `Diagnostic`, `Severity`, `FileLocation`, `Config`, SKILL.md parsing (frontmatter, sections, body extraction), file discovery, state-name validation.

**Does NOT own:** loop contracts, transitions, graph, simulation, best-practices rules, CLI.

### 2.2 `skill-loop` — loop-aware validation

**Owns:** `LoopContract`, `TransitionRule`, enforced states, graph construction (from core's transitions), all loop validators (graph, simulation, state consistency, loop language, loop sections, loop completeness, loop state files, cross-references, constraints), deskcheck sub-pattern validator.

**Depends on:** `skill-core` only.

**Forge conventions live here:** LOOP.md structure, canonical sections, enforced state vocabulary, L1-RIGID / L2-GUIDED levels.

**If a skill has no transitions:** `skill-loop` produces zero diagnostics (no-op). The skill passes loop validation trivially.

### 2.3 `skill-typecheck` — CLI + best-practices

**Owns:** CLI (clap), JSON output formatter, all best-practices validators (frontmatter, naming, structure, progressive disclosure, terminology, workflow, anti-patterns).

**Depends on:** `skill-core` and `skill-loop`.

**Pipeline:** calls `skill-loop::validate_all()` and `best_practices::check_all()`, merges diagnostics, formats output.

### 2.4 Dependency graph

```
skill-core
    ↑
    ├── skill-loop (uses core types + parsers)
    └── skill-typecheck (uses core types + parsers + loop validators)
```

`skill-loop` never depends on `skill-typecheck`, and vice versa. The CLI is the only crate that combines both layers.

---

## 3. Enforced Language

The tool ships with a built-in default vocabulary. Every project can override it with a `.skill-typecheck.yaml` file at the skills directory root.

### 3.1 Built-in defaults

```yaml
# Shipped in skill-loop, overridable per project

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

halt_reasons:
  - stall
  - ambiguous
  - human-gate
  - unsafe
  - budget

canonical_loop_sections:
  - Entry Conditions
  - Loop State Schema
  - Single Iteration Step
  - Proof of Progress
  - State Transition Rule
  - Halt Conditions
  - Handoff Target

canonical_skill_sections:
  - Description
  - Rules
  - State Model
  - Entry Conditions
  - Halt Conditions

state_model_aliases:
  - State Model
  - The Loop
  - Loop States
  - States

enforced_states:
  - backlog
  - in-analysis
  - ready-for-dev
  - in-dev
  - ready-for-qa
  - in-qa
  - ready-for-acceptance
  - in-acceptance
  - ready-to-deploy
  - halted-stall
  - halted-human-gate
  - halted-unsafe
```

### 3.2 Deskcheck — conditional sub-pattern

Deskcheck states (`ready-for-deskcheck`, `in-deskcheck`) are NOT in the enforced list. If they appear in any transition or State Model, the verifier enforces the full deskcheck subgraph:

```
in-dev → ready-for-deskcheck → in-deskcheck → (in-dev | ready-for-qa)
```

Required exits:

| Condition | Required transition |
|---|---|
| `ready-for-deskcheck` exists | Must have `→ in-deskcheck` |
| `in-deskcheck` exists | Must have `→ in-dev` (feedback loop) |
| `in-deskcheck` exists | Must have `→ ready-for-qa` (completion path) |

### 3.3 Bug-feedback transitions

Bug severity determines the feedback destination:

| Bug found in | Goes back to | Diagnostic if missing |
|---|---|---|
| `in-deskcheck` | `in-dev` | `state-missing-deskcheck-feedback` |
| `in-qa` | `ready-for-dev` | `state-missing-bug-feedback` |
| `in-acceptance` | `ready-for-dev` | `state-missing-bug-feedback` |

`in-qa` and `in-acceptance` must each have a path to `ready-for-dev`. This represents the formal bug-report flow: a bug found in QA or acceptance gets a separate bug report and re-enters the backlog queue at `ready-for-dev`.

### 3.4 Configuration merge

```
Built-in default → Project .skill-typecheck.yaml → Final config
```

Project config wins per field. Unspecified fields fall back to defaults.

---

## 4. What the Loop Verifier Proves

With the enforced language applied, `skill-loop` proves:

1. **Vocabulary conformance** — every transition verb is in `standard_verbs`, every halt reason is in `halt_reasons`. Violation = Error.
2. **State declaration** — every transition endpoint is declared in at least one State Model section (and reverse: every graph node is declared in prose).
3. **Graph completeness** — every declared state appears in the graph (forward), and every graph node is declared in prose (reverse). Bi-directionally complete.
4. **Graph soundness** — terminals are sinks, no non-terminal dead-ends, no unreachable non-terminal states, all entry points are reachable.
5. **Enforced states present** — every state in `enforced_states` appears in the graph and is owned by at least one skill.
6. **Required exits present** — states that represent decision gates (`in-qa`, `in-acceptance`, `in-deskcheck`) have their required outbound transitions.
7. **Deskcheck subgraph** — if deskcheck states exist, the full sub-pattern is complete (conditional enforcement).
8. **Self-loops detected** — states that only transition to themselves are flagged.

---

## 5. Best-Practices Checks

22 rules across 7 categories. All diagnostics emitted by `skill-typecheck`.

### 5.1 Frontmatter (structural → Error unless noted)

| Code | Rule | Severity |
|---|---|---|
| `skill-missing-name` | `name` field missing from frontmatter | Error |
| `skill-name-too-long` | `name` exceeds 64 characters | Error |
| `skill-name-invalid-chars` | `name` contains non-`[a-z0-9-]` characters | Error |
| `skill-name-reserved-word` | `name` contains `anthropic` or `claude` | Error |
| `skill-missing-description` | `description` field missing | Error |
| `skill-description-too-long` | `description` exceeds 1024 characters | Error |
| `skill-description-xml-tag` | `description` contains XML tags | Error |
| `skill-description-not-third-person` | `description` uses first/second person (`I can`, `You can`) | Warning |

### 5.2 Naming conventions (style → Warning)

| Code | Rule | Severity |
|---|---|---|
| `skill-name-not-gerund` | Name not in gerund form (`-ing`) | Warning |
| `skill-name-vague` | Name is `helper`, `utils`, `tools`, `misc` | Warning |
| `skill-naming-inconsistent` | Project mixes gerund/noun/action patterns across skills | Warning |

### 5.3 Structure (provable → Error unless noted)

| Code | Rule | Severity |
|---|---|---|
| `skill-body-too-long` | SKILL.md body exceeds 500 lines | Error |
| `skill-deep-reference` | Reference chain deeper than one level (A → B → C) | Warning |
| `skill-ref-missing-toc` | Reference file >100 lines without table of contents | Warning |
| `skill-windows-path` | Windows-style path (`\`) in file references | Error |
| `skill-time-sensitive` | Time-sensitive language detected in content | Warning |

### 5.4 Progressive disclosure (style → Warning)

| Code | Rule | Severity |
|---|---|---|
| `skill-no-progressive-disclosure` | SKILL.md >200 lines with no separate reference files | Warning |
| `skill-orphan-reference` | Reference file in skill directory not linked from SKILL.md | Warning |

### 5.5 Terminology consistency (heuristic → Warning)

| Code | Rule | Severity |
|---|---|---|
| `skill-term-inconsistency` | Multiple terms used for the same concept within one skill | Warning |

### 5.6 Workflow quality (heuristic → Warning)

| Code | Rule | Severity |
|---|---|---|
| `skill-missing-checklist` | Multi-step workflow without checklist or numbered steps | Warning |
| `skill-missing-feedback-loop` | No `validate → fix → repeat` pattern in workflow | Warning |
| `skill-missing-conditionals` | Branching logic without explicit conditional structure | Warning |

### 5.7 Anti-patterns (heuristic → Warning unless noted)

| Code | Rule | Severity |
|---|---|---|
| `skill-too-many-options` | Three or more equivalent approaches for the same task | Warning |
| `skill-magic-numbers` | Undocumented magic numbers in script code | Warning |
| `skill-punts-to-claude` | `try: ... except:` with bare `raise` (no error handling) | Warning |

---

## 6. CLI

### 6.1 Usage

```bash
skill-typecheck path/to/skills/              # text output
skill-typecheck path/to/skills/ --json       # machine-readable
```

No flags for what to check. Default run = everything.

### 6.2 Exit codes

| Code | Meaning |
|---|---|
| 0 | Zero diagnostics of any severity |
| 1 | At least one Error |
| 2 | Internal error (parse failure, file not found) |

Warnings alone do NOT cause non-zero exit. All loop/graph issues are Errors, so any graph problem → exit 1.

### 6.3 JSON output

```json
{
  "skills_checked": 21,
  "diagnostics": [
    {
      "code": "loop-nonstandard-verb",
      "severity": "Error",
      "skill": "facilitating-inception",
      "file": "skills/meta/facilitating-inception/LOOP.md",
      "line": 14,
      "message": "Verb 'invoke' is not in standard verbs: [trigger, handoff, halt, ...]"
    }
  ],
  "summary": {
    "errors": 3,
    "warnings": 15
  }
}
```

### 6.4 Text output

```
skills/meta/facilitating-inception/LOOP.md:14  Error    loop-nonstandard-verb  Verb 'invoke' is not in standard verbs
skills/development/running-tdd-loops/SKILL.md:3 Error    skill-name-reserved-word  Name contains 'anthropic'

21 skills checked. 3 error(s), 15 warning(s).
```

---

## 7. Gap Fixes From Prior Analysis

This design closes every high-severity gap identified across 4 rounds of analysis.

| Gap | Source | Fix location |
|---|---|---|
| ASCII `->` not recognized as transition arrow | R4-C1 | `skill-core` parser accepts both `→` (Unicode) and `->` (ASCII) |
| `extract_section_body` vs `parse_sections` mismatch (formatted headings) | R4-C2 | Use pulldown_cmark byte offsets for body extraction |
| `## State Model` hardcoded; aliases ignored | R2-R3, R3-5 | Config-driven `state_model_aliases` list |
| No enforced-state validation | R3-Req3 | `enforced_states` config + `validate_enforced_states` |
| Transition endpoints not validated against declared states | R3-Req1 | Endpoint → declared-state check |
| Reverse graph completeness missing | R2-R3, R3-Req2 | Graph node → prose state check |
| Dead `TransitionToUnknownState` in simulation | R3-Req1 | Removed or fixed; replaced by endpoint validation |
| `loop-nonstandard-verb` false positives on compound verbs | R4 | Compound verb merging (`Hand off` → `handoff`) |
| `state-undefined-in-graph` flags skill names as states | R4 | Tighter `is_state_like` heuristic |
| Halt reason regex misses `halt;` pattern | R4 | Regex broadened or prose scan added |
| Missing `name` in frontmatter silently drops skill | R4-C4 | Error diagnostic instead of silent skip |
| `has_all_canonical_sections` duplicate bug | R4 | Fix duplicate counting |
| `halt stall after XYZ iterations` → `halt_after = Some(0)` | R4 | Fix fallback to `None` when parse fails |
| Hardcoded `20` for `max_iterations` | R3-Req5 | Use `Config.max_iterations` |
| Skill backtick-quoted states dropped | R4-C3 | Parser strips backticks before state-name regex |

---

## 8. Migrating From `skill-loop-verifier`

| Current path | New path |
|---|---|
| `skill-loop-verifier/src/types.rs` | Split: universal types → `skill-core`, loop types → `skill-loop` |
| `skill-loop-verifier/src/parser/` | `skill.rs`, `yaml.rs` → `skill-core`; `handoff.rs`, `loop_.rs` → `skill-loop` |
| `skill-loop-verifier/src/repo.rs` | Split: discovery → `skill-core`, graph build → `skill-loop` |
| `skill-loop-verifier/src/config.rs` | → `skill-core` (unified `Config`) |
| `skill-loop-verifier/src/diagnostic.rs` | → `skill-core` |
| `skill-loop-verifier/src/validators/` | → `skill-loop` |
| `skill-loop-verifier/src/simulation/` | → `skill-loop` |
| `skill-loop-verifier/src/lib.rs` | Becomes `skill-typecheck/src/main.rs` |
| Tests | Split by crate. Existing tests → `skill-loop/tests/`. New tests → `skill-typecheck/tests/`. |

---

## 9. Non-Goals

- Online / web service. This is a CLI tool.
- Writing or fixing skills. It only checks.
- Coq/OCaml formal verification. Pure Rust.
- Plugin system. Rules are hardcoded validators, not plugins.
- Skill generation or scaffolding. Check-only.
- Performance optimization for repos with >1000 skills. Correctness first.

---

## 10. Success Criteria

1. Running against `forge/skills/` produces zero Errors (all loop violations are already fixed; best-practices Warnings may remain).
2. Running against a deliberately malformed skill repo catches every violation the tool is designed to catch.
3. Exit code 1 for any structural/syntax violation.
4. JSON output is parseable and includes file/line/skill for every diagnostic.
5. The tool can check an arbitrary directory of skills with no forge-specific configuration.

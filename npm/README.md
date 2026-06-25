<p align="center">
  <img src="https://raw.githubusercontent.com/loopworx/loopkit/main/assets/loopkit-icon.png" alt="loopkit" width="200">
</p>

<h1 align="center">loopkit</h1>

<p align="center">
  <strong>The Loop Language compiler.</strong> loopkit verifies that your agent skills follow the Loop Language — a formal contract language for agent behavior.
</p>

<p align="center">
  Skills written in plain markdown don't give agents clear instructions. Skills written in the Loop Language define what the agent should do, how it proves progress, when it stops, and who it hands off to — all machine-verifiable before anything hits production.
</p>

<p align="center">
  <a href="https://github.com/loopworx/loopkit/releases">Download</a> |
  <a href="#quick-start">Quick Start</a> |
  <a href="docs/loop-language.md">Docs</a>
</p>

```bash
cargo install loopkit
loopkit /path/to/project --verbose
```

---

## Why a contract language?

Prompt files and unstructured skill documents fail in predictable ways: agents skip steps, forget handoffs, loop forever, use inconsistent terminology, or make architectural decisions they shouldn't. Loop Language skills are structured: every skill declares its **state machine** (the set of states it moves through), its **transition rules** (when and why it changes state), its **halt conditions** (when to stop and escalate), and its **handoff targets** (who owns the next step). These aren't suggestions. They're contracts. loopkit enforces them.

A skill without a state machine is documentation. A skill with a verifiable Loop Language contract is **executable process**.

---

## What loopkit verifies

loopkit runs 20 validators organized under two engines:

### The Graph Engine — state machine integrity

| Validator | Contract ensures... |
|---|---|
| `graph` | No dead-end states, no unreachable nodes, no self-loop-only traps |
| `simulation` | Every entry state can reach a terminal within budget |
| `loop_language` | All verbs and halt reasons are declared vocabulary |
| `loop_sections` | LOOP.md has all required sections in canonical order |
| `state_consistency` | Every state in prose is in the graph, every graph node is declared |
| `enforced_states` | Every configured mandatory state appears somewhere |
| `deskcheck` | Desk check states have proper entry/feedback/forward edges |
| `bug_feedback` | QA and acceptance states feed bugs back to development |
| `loop_completeness` | Skill level (L1/L2/L3) has matching section requirements |
| `loop_state_files` | External state files referenced by the contract exist |
| `cross_references` | Every `handoff <skill>` points to a real skill |
| `constraints` | Handoff targets follow configured routing rules |

### The Style Engine — best practices

| Validator | Checks |
|---|---|
| `naming` | Gerund names, consistent patterns across skills |
| `frontmatter` | Description exists, name is kebab-case, no reserved words |
| `structure` | Size limits, time-sensitive language, platform paths |
| `terminology` | Terms used consistently across all skill files |
| `anti_patterns` | Undocumented magic numbers, option-bombing |
| `progressive` | Long skills reference supplementary files |
| `workflow` | Workflow skills include checklist sections |
| `scripts` | Bundled scripts have dependency declarations |

---

## Quick Start

```bash
# Install
cargo install loopkit

# Check a project
loopkit /path/to/project

# Verbose — see per-validator counts
loopkit /path/to/project --verbose

# JSON — for CI machines
loopkit /path/to/project --json
```

Output:
```
skills/running-tdd-loops/SKILL.md    Error    name-non-gerund      Skill name not a gerund (-ing form)
skills/writing-stories/LOOP.md       Warning  loop-nonstandard-verb "flurbish" is not a standard verb

21 skills checked. 1 error(s), 2 warning(s).
```

---

## What a Loop Language skill looks like

```
my-project/
├── .loopkit.yaml          # vocabulary, enforced states, validator config
└── skills/
    └── running-tdd-loops/
        ├── SKILL.md       # metadata + body
        └── LOOP.md        # the contract
```

### SKILL.md — identity

```yaml
---
name: running-tdd-loops
description: Execute TDD inner loops (FE component + BE CDC contract tests)
metadata:
  category: development
---

## Description
...

## Rules
...

## State Model
This skill operates across `in-dev`, `in-deskcheck`, and `halted-stall`.
```

### LOOP.md — the contract

```markdown
## Entry Conditions
Story is in-dev with clear ACs

## Loop State Schema
| field | type | description |
|-------|------|-------------|
| story | str  | Story identifier |
| pass  | bool | Current test status |

## Single Iteration Step
1. write failing test
2. implement minimum code
3. confirm green
4. refactor

## Proof of Progress
`cargo test` output shows all green

## State Transition Rule
transition in-dev → in-deskcheck
  trigger all ACs implemented and green
  handoff running-desk-checks to qa-agent

transition in-dev → halted-stall
  halt stall after 10 iterations

## Halt Conditions
halt stall after 10 iterations

## Handoff Target
handoff running-desk-checks to qa-agent
```

Every section is machine-parsed. The transition rules are extracted into a graph. The handoff target is cross-referenced. The halt condition is validated against declared vocabulary. Nothing is free-form prose — it's all contract.

---

## `.loopkit.yaml` reference

loopkit is fully config-driven. Zero state names are hardcoded. You define the vocabulary for your project.

```yaml
# Where skills live
skills_dir: skills/
max_iterations: 20

# Declared vocabulary — loopkit flags anything outside this set
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

# Required LOOP.md sections (checked for presence and order)
canonical_loop_sections:
  - Entry Conditions
  - Loop State Schema
  - Single Iteration Step
  - Proof of Progress
  - State Transition Rule
  - Halt Conditions
  - Handoff Target

# Required SKILL.md sections
canonical_skill_sections:
  - Description
  - Rules
  - State Model

# Alternate headings for the state model section
state_model_aliases:
  - State Model
  - The Loop
  - Loop States

# States that MUST appear in at least one transition
enforced_states:
  - name: in-dev
    agent: developer-agent
    description: Developer builds story AC by AC
  - name: in-qa
    agent: qa-agent
    description: QA runs full regression suite
  - name: done
    agent: ""
    description: Story deployed and verified
  - name: halted-stall
    agent: ""
    description: No progress for N iterations

# Opt-in: desk check pattern validation
deskcheck_enabled: true
deskcheck_state: in-deskcheck
deskcheck_entry_from: in-dev
deskcheck_feedback_to: in-dev
deskcheck_forward_to: in-qa

# Opt-in: bug feedback loop validation
bug_feedback_enabled: true
bug_feedback_qa_state: in-qa
bug_feedback_acceptance_state: in-acceptance
bug_feedback_return_to: in-dev
```

---

## Concepts

### The Loop Language

The Loop Language is a set of conventions that turn agent skill documents into **verifiable contracts**:

1. **State machines** — every skill operates across a finite set of named states
2. **Transition rules** — state changes have triggers, handoffs, and halt conditions
3. **Declared vocabulary** — verbs, halt reasons, and state names are all explicit
4. **Proof of progress** — every iteration declares what "done for this round" means
5. **Handoff integrity** — every transition knows who owns the next step

Skills that follow the Loop Language can be **statically verified** before an agent ever runs them. loopkit is the verifier.

### Why not just a linter?

Linters check style. loopkit checks **behavior**. A missing handoff target isn't a formatting issue — it means your agent will dead-end in a state with no path out. An undeclared state in prose means your graph has nodes the agent can reach but never describe. An unenforced state means you claimed it exists but nothing transitions to or from it. These are logical errors that break multi-agent coordination.

---

## Documentation

- [The Loop Language](docs/loop-language.md) — contract language for agent skills: states, transitions, halts, handoffs
- [`.loopkit.yaml` Reference](docs/config-reference.md) — every config field, default values, design rationale

---

## Development

```bash
git clone https://github.com/loopworx/loopkit
cd loopkit
cargo build
cargo test
cargo run -p loopkit -- examples/test-fixture --verbose
```

```
crates/
├── loopkit-core/       # Config, discovery, parser, diagnostics
├── loopkit-graph/      # Graph engine, validators, simulation
└── loopkit/            # CLI, style engine, acceptance tests
```

---

## License

MIT

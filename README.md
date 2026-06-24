# loopkit

Static analysis for agent skill contracts. Validates that your skills follow the [agentskills.io](https://agentskills.io) specification and are production-ready for multi-agent collaboration.

```bash
cargo install loopkit
loopkit /path/to/your-project --verbose
```

[![Rust](https://img.shields.io/badge/rust-1.82+-orange)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

---

## What it checks

Loopkit runs 20 validators across two categories:

### Graph Validators (12)
| Validator | What it checks |
|---|---|
| `graph` | State machine integrity — dead ends, unreachable states, self-loops |
| `simulation` | Budget-constrained reachability from entry to terminal states |
| `loop_language` | Standard verbs, halt reasons, transition syntax |
| `loop_sections` | Required LOOP.md sections in correct order |
| `state_consistency` | Bidirectional match between declared and graph states |
| `enforced_states` | Every `.loopkit.yaml` enforced state appears in transitions |
| `deskcheck` | Desk check transition pattern (opt-in, configurable) |
| `bug_feedback` | Bug feedback loops from QA/acceptance back to dev (opt-in, configurable) |
| `loop_completeness` | Level-appropriate LOOP.md + SKILL.md completeness |
| `loop_state_files` | Referenced loop state files exist |
| `cross_references` | Handoff targets and README skill references resolve |
| `constraints` | Skill-to-skill handoff constraints |

### Best Practices (8)
| Validator | What it checks |
|---|---|
| `naming` | Gerund naming, consistent patterns, no vague names |
| `frontmatter` | Description exists, name is kebab-case, no reserved words |
| `structure` | Line count limits, time-sensitive language, Windows paths |
| `terminology` | Consistent term usage across skills |
| `anti_patterns` | Magic numbers, too-many-options, undocumented magic |
| `progressive` | Long skills should reference supplementary files |
| `workflow` | Checklist sections in workflow skills |
| `scripts` | Bundled scripts awareness, inline deps, no help scripts |

---

## Quick Start

### Install

```bash
# From source
cargo install loopkit

# Or clone and build
git clone https://github.com/loopworx/loopkit
cd loopkit
cargo build --release
./target/release/loopkit --help
```

### Run

```bash
# Basic check
loopkit /path/to/project

# Verbose mode — see per-validator diagnostic counts
loopkit /path/to/project --verbose

# JSON output for CI
loopkit /path/to/project --json
```

Output:
```
/Users/me/project/skills/running-tdd-loops/SKILL.md    Error    state-missing-checkpoint    Missing progress checkpoint
/Users/me/project/skills/writing-stories/SKILL.md       Warning  name-non-gerund            Skill name not a gerund

21 skills checked. 1 error(s), 2 warning(s).
```

---

## Project Layout

loopkit expects the agentskills.io flat layout:

```
my-project/
├── .loopkit.yaml          # loopkit config (required for enforced states etc.)
├── README.md              # project readme (checked for skill references)
└── skills/
    ├── running-tdd-loops/
    │   ├── SKILL.md        # skill metadata + body
    │   └── LOOP.md         # loop contract (state machine, transitions)
    ├── writing-stories/
    │   ├── SKILL.md
    │   └── LOOP.md
    └── ...
```

Each skill directory must contain at least `SKILL.md`. `LOOP.md` is optional but required for stateful skills.

### SKILL.md

Frontmatter (YAML) + body content:

```markdown
---
name: running-tdd-loops
description: Execute TDD inner loops (FE component + BE CDC contract)
metadata:
  category: development
---

## Description
...

## Rules
...
```

### LOOP.md

The loop contract defines the skill's state machine:

```markdown
## Entry Conditions
Story is in-dev with clear ACs

## Loop State Schema
| field | type | description |
|-------|------|-------------|
| story | str  | Story identifier |

## Single Iteration Step
1. Write failing test
2. Implement code
3. Confirm green
4. Refactor

## Proof of Progress
`cargo test` all green

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

---

## Configuration — `.loopkit.yaml`

All project-specific configuration lives in `.loopkit.yaml` at the project root. loopkit is fully agnostic — no state names are hardcoded in the binary.

### Full reference

```yaml
# Where skills live (default: skills/)
skills_dir: skills/

# Max iterations for budget-constrained simulation (default: 20)
max_iterations: 20

# Standard verbs recognized in LOOP.md steps
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
  # ... add project-specific verbs

# Standard halt reasons
halt_reasons:
  - stall
  - ambiguous
  - human-gate
  - unsafe
  - budget

# Required LOOP.md sections (in order)
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

# Alternative heading names for the State Model section
state_model_aliases:
  - State Model
  - The Loop
  - Loop States
  - States

# Enforced states — every state here must appear in at least one transition
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

### Why no defaults for enforced states?

loopkit is **fully agnostic**. It ships with zero hardcoded state names. Every state, transition, and validation pattern is driven by your `.loopkit.yaml`. This means:

- Use whatever state taxonomy fits your team — `in-progress` vs `in-dev`, `review` vs `deskcheck`, any terminal state name
- Desk check and bug feedback validators are **opt-in** — enable them when your process needs them, configure them with your state names
- Cross-reference exceptions (states that shouldn't trigger "unknown skill" warnings) are derived from your `enforced_states` and `halt_reasons`

---

## CI Integration

```yaml
# .github/workflows/loopkit.yml
name: loopkit
on: [push, pull_request]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo install loopkit
      - run: loopkit . --json > loopkit-report.json
      - uses: actions/upload-artifact@v4
        with:
          name: loopkit-report
          path: loopkit-report.json
```

---

## Verified Projects

| Project | Skills | Result |
|---|---|---|
| [Forge](https://github.com/yaman/forge) | 21 | 0 errors, 0 warnings |

---

## Development

```bash
git clone https://github.com/loopworx/loopkit
cd loopkit

# Build
cargo build

# Run tests
cargo test

# Run against test fixture
cargo run -p loopkit -- examples/test-fixture --verbose
```

### Architecture

```
crates/
├── loopkit-core/       # Config, discovery, parser, diagnostics
├── loopkit-graph/      # Graph construction, validators, simulation
└── loopkit/            # CLI, best-practice validators, acceptance tests
```

### Adding a validator

1. Create `crates/loopkit-graph/src/validators/my_validator.rs` with a `validate()` function
2. Register it in `crates/loopkit-graph/src/validators/mod.rs`
3. Add tests
4. If config-driven, add fields to `Config` in `crates/loopkit-core/src/types.rs`

---

## License

MIT

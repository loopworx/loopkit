# The Loop Language

The Loop Language is a contract language for agent skills. It turns markdown documents into verifiable state machines. Every skill declares what states it moves through, when it transitions, how it proves progress, when it halts, and who it hands off to.

## Why contracts, not prose

Prompt files and prose skill documents fail in predictable ways:

- Agents skip steps because the document was ambiguous
- Agents forget handoffs and dead-end in a state
- Agents loop forever because halt conditions weren't declared
- Agents use inconsistent terminology, confusing other agents
- Agents make decisions they shouldn't because role boundaries weren't enforced

The Loop Language fixes this by requiring every skill to declare its behavior as a machine-readable contract. loopkit verifies these contracts before any agent runs them.

## The anatomy of a Loop Language skill

Every skill has two files:

### SKILL.md — identity and instructions

Frontmatter declares the skill's name and metadata. Required sections define what the agent actually does.

```markdown
---
name: running-tdd-loops
description: Execute TDD inner loops (FE component + BE CDC contract tests)
metadata:
  category: development
---

## Description
Execute one TDD cycle per acceptance criterion sub-slice...

## Rules
1. Never write implementation code before a failing test
2. Test one behavior at a time
3. Refactor only on green

## State Model
This skill operates across `in-dev`, `in-deskcheck`, `in-qa`,
`in-acceptance`, `done`, and `halted-stall`.
```

**Required sections** (configurable in `.loopkit.yaml`):
- `Description` — what the skill does, used by the agent to decide when to invoke it
- `Rules` — non-negotiable constraints the agent must follow
- `State Model` — declares all states this skill moves through (or one of the aliases)

**Frontmatter requirements:**
- `name` — lowercase kebab-case, matching the directory name
- `description` — 10–200 characters, no XML, no first person
- `metadata.category` — optional category for organization

### LOOP.md — the contract

The LOOP.md is the skill's state machine contract. Every section is machine-parsed.

```markdown
## Entry Conditions
Story is in-dev with clear ACs and development environment ready

## Loop State Schema
| field    | type | description            |
|----------|------|------------------------|
| story    | str  | Story identifier       |
| ac_index | int  | Current AC index       |
| pass     | bool | Current test status    |

## Single Iteration Step
1. write the failing test for current AC
2. run the test and confirm failure
3. implement the minimum code
4. run the test and confirm green
5. refactor while tests stay green
6. commit

## Proof of Progress
`cargo test` output shows all tests green for the current AC

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

**Required sections** (in canonical order):
1. **Entry Conditions** — when the agent can start this skill's loop
2. **Loop State Schema** — state variables tracked across iterations
3. **Single Iteration Step** — the numbered steps for one iteration
4. **Proof of Progress** — objective evidence the iteration accomplished something
5. **State Transition Rule** — when and why the state changes
6. **Halt Conditions** — when to stop and escalate
7. **Handoff Target** — who owns the next step

## Transition syntax

```
transition <from> → <to>
  trigger <condition>
  handoff <skill> to <agent>

transition <from> → <halted-*>
  halt <reason> after <N> iterations
```

- Arrow can be `→`, `->`, or `--->`
- States can be backtick-quoted: `` `in-dev` ``
- Multiple transitions per section are supported
- `trigger` and `handoff` are optional
- `halt after N` is optional (defaults to unbounded)

## State name conventions

State names follow kebab-case with at least one hyphen:

```
in-dev           ✓
in-deskcheck     ✓
in-qa            ✓
in-acceptance    ✓
halted-stall     ✓
done             ✓ (allowed if declared in enforced_states)
backlog          ✗ (no hyphen, not enforced)
In-Dev           ✗ (uppercase)
in.progress      ✗ (dots)
```

The exception for no-hyphen states (like `done`) is only granted when the state appears in `.loopkit.yaml`'s `enforced_states` — loopkit derives this from config, not from any hardcoded list.

## Skill naming conventions

Skill names use gerund (verb-ing) form:

```
running-tdd-loops       ✓
writing-stories         ✓
facilitating-inception  ✓
deciding-architecture   ✓
tdd-loops               ✗ (not gerund)
story-writer            ✗ (not gerund)
```

Consistency matters. If most skills are gerunds, all skills should be gerunds. loopkit warns on mixed patterns.

## Halt reasons

Halt reasons are declared in `.loopkit.yaml`. A transition that halts with an undeclared reason emits a warning.

```yaml
halt_reasons:
  - stall         # no progress after N iterations
  - ambiguous     # conflicting or unclear state
  - human-gate    # requires human decision
  - unsafe        # unsafe condition detected
  - budget        # cost or time budget exceeded
```

## Standard verbs

Verbs used in numbered steps are validated against the declared vocabulary:

```yaml
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
  # ... add domain-specific verbs
```

A step like `1. flurbish the widget` will warn if `flurbish` is not in the verb list.

## Agent handoffs

Every transition can specify a handoff:

```
transition in-dev → in-deskcheck
  trigger all ACs green
  handoff running-desk-checks to qa-agent
```

The format is `handoff <skill> to <agent>`. loopkit cross-references the skill name against the skills directory. Unknown targets emit a warning.

The special handoff target `done` indicates the story is finished and requires no further skill invocation.

## State model declaration

Every SKILL.md must declare which states the skill operates across. This can appear under any of:

```yaml
state_model_aliases:
  - State Model
  - The Loop
  - Loop States
  - States
```

The body of this section lists state names (usually backtick-quoted):

```markdown
## State Model
This skill moves between `in-dev`, `in-deskcheck`, `in-qa`, and `halted-stall`.
```

loopkit verifies bidirectional consistency: every state in the prose must appear in the graph, and every graph node must be declared in some skill's state model.

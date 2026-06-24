# loopkit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the flat `skill-loop-verifier` crate into a three-crate workspace (`loopkit-core`, `loopkit-graph`, `loopkit`) with expanded Config, enforced-state validation, deskcheck/bug-feedback pattern checks, 22 best-practices rules, and all gap-analysis fixes.

**Architecture:** Workspace with layered dependencies — `loopkit-graph` depends on `loopkit-core`; `loopkit` (CLI) depends on both. `loopkit-core` holds universal types, parsers, and discovery. `loopkit-graph` holds loop-specific validation (graph, simulation, states, vocabulary). `loopkit` holds the CLI and best-practices validators.

**Tech Stack:** Rust 2021 edition, pulldown-cmark 0.13 (body extraction using byte offsets), serde/serde_yml for config, clap for CLI, regex 1.10, walkdir 2.5.

---

## Phase 0: Workspace Scaffold

### Task 0.1: Create workspace structure

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/loopkit-core/Cargo.toml`
- Create: `crates/loopkit-core/src/lib.rs`
- Create: `crates/loopkit-graph/Cargo.toml`
- Create: `crates/loopkit-graph/src/lib.rs`
- Create: `crates/loopkit/Cargo.toml`
- Create: `crates/loopkit/src/main.rs`
- Modify: `.loopkit.yaml` (move to workspace root, rename from `.loop-verifier.yaml`)

- [ ] **Step 1: Write workspace root Cargo.toml**

```toml
# /Cargo.toml (workspace root)
[workspace]
members = ["crates/loopkit-core", "crates/loopkit-graph", "crates/loopkit"]
resolver = "2"
```

- [ ] **Step 2: Write loopkit-core Cargo.toml**

```toml
# crates/loopkit-core/Cargo.toml
[package]
name = "loopkit-core"
version = "0.3.0"
edition = "2021"

[dependencies]
pulldown-cmark = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_yml = "0.0.13"
regex = "1.10"
walkdir = "2.5"
serde_json = "1.0"
thiserror = "2.0"

[dev-dependencies]
tempfile = "3.10"
```

- [ ] **Step 3: Write loopkit-graph Cargo.toml**

```toml
# crates/loopkit-graph/Cargo.toml
[package]
name = "loopkit-graph"
version = "0.3.0"
edition = "2021"

[dependencies]
loopkit-core = { path = "../loopkit-core" }
pulldown-cmark = "0.13"
regex = "1.10"

[dev-dependencies]
tempfile = "3.10"
```

- [ ] **Step 4: Write loopkit CLI Cargo.toml**

```toml
# crates/loopkit/Cargo.toml
[package]
name = "loopkit"
version = "0.3.0"
edition = "2021"

[[bin]]
name = "loopkit"
path = "src/main.rs"

[dependencies]
loopkit-core = { path = "../loopkit-core" }
loopkit-graph = { path = "../loopkit-graph" }
clap = { version = "4", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
tempfile = "3.10"
```

- [ ] **Step 5: Write stub lib.rs files**

```rust
// crates/loopkit-core/src/lib.rs
pub mod config;
pub mod diagnostic;
pub mod discovery;
pub mod parser;
pub mod types;
```

```rust
// crates/loopkit-graph/src/lib.rs
pub mod graph;
pub mod parser;
pub mod simulation;
pub mod types;
pub mod validators;
```

```rust
// crates/loopkit/src/main.rs
fn main() {
    println!("loopkit v0.3.0");
}
```

- [ ] **Step 6: Verify workspace builds**

```bash
cargo build
```

Expected: `Finished dev [unoptimized + debuginfo] target(s)` — three crates compile clean.

- [ ] **Step 7: Remove old flat-crate files**

```bash
rm Cargo.toml src/lib.rs src/main.rs src/types.rs src/config.rs src/diagnostic.rs src/repo.rs src/parser/mod.rs src/parser/skill.rs src/parser/loop_.rs src/parser/handoff.rs src/parser/yaml.rs src/validators/mod.rs src/validators/graph.rs src/validators/state_consistency.rs src/validators/skill_completeness.rs src/validators/loop_language.rs src/validators/loop_sections.rs src/validators/loop_state_files.rs src/validators/cross_references.rs src/validators/constraints.rs src/simulation/mod.rs .loopkit.yaml
rmdir src/parser src/validators src/simulation src
```

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "scaffold: three-crate workspace (loopkit-core, loopkit-graph, loopkit)"
git push
```

---

## Phase 1: loopkit-core — Universal Types, Parsing, Discovery

### Task 1.1: Port universal types from git history

**Files:**
- Create: `crates/loopkit-core/src/types.rs`

- [ ] **Step 1: Extract types from old commit (e2c8106~1 is before extraction)**

```bash
git show e2c8106~1:src/types.rs > /tmp/old_types.rs
```

- [ ] **Step 2: Write loopkit-core types — only universal parts**

Port these structs/enums from old `types.rs` (strip loop-specific state machine types):
- `Config` with `skills_dir` (default `"skills/"`), `max_iterations` (default `20`)
- `Severity` enum: `Error`, `Warning`, `Info`
- `Diagnostic` struct with `severity`, `code`, `message`, `location: FileLocation`, `help`
- `FileLocation` struct with `path: PathBuf`, `line: Option<u32>`, `column: Option<u32>`
- `Skill` struct with `name`, `level`, `owner`, `description`, `category`, `path`, `skill_md: PathBuf`, `sections: Vec<Section>`, `states: Vec<String>`
- `Section` struct with `name: String`, `body: String`

```rust
// crates/loopkit-core/src/types.rs
// SKIP the old types::Config with just skills_dir+max_iterations.
// We use the expanded Config below (Task 1.5).
```

- [ ] **Step 3: Write Diagnostic constructors**

```rust
impl Diagnostic {
    pub fn error(code: &str, message: String, path: PathBuf) -> Self { ... }
    pub fn warning(code: &str, message: String, path: PathBuf) -> Self { ... }
    pub fn info(code: &str, message: String, path: PathBuf) -> Self { ... }
    pub fn at_line(self, line: u32) -> Self { self.with_line(Some(line)) }
    pub fn at_span(self, line: u32, column: u32) -> Self { ... }
}
impl FileLocation {
    pub fn new(path: PathBuf) -> Self { ... }
    pub fn at(path: PathBuf, line: u32, column: u32) -> Self { ... }
}
```

- [ ] **Step 4: Commit**

### Task 1.2: Port SKILL.md parser from git history

**Files:**
- Create: `crates/loopkit-core/src/parser/mod.rs`
- Create: `crates/loopkit-core/src/parser/skill.rs`

- [ ] **Step 1: Extract old parser from git**

```bash
git show e2c8106~1:src/parser/skill.rs > /tmp/old_parser_skill.rs
git show e2c8106~1:src/parser/mod.rs > /tmp/old_parser_mod.rs
```

- [ ] **Step 2: Write `parse_frontmatter`**

Port the YAML frontmatter parser. Returns `HashMap<String, String>` and body start line. Keep existing logic.

- [ ] **Step 3: Write `parse_sections`**

Port the pulldown_cmark-based section parser. Only H2 headings (`Heading` with `H2`). Each section has `name: String` and `body: String`.

- [ ] **Step 4: Write `extract_section_body` — FIX gap R4-C2**

Use pulldown_cmark byte offsets instead of raw `str::find`. The old code used `content.find("## Some Title")` which breaks on formatted headings (`## **Title**`, `## \`Title\``).

```rust
pub fn extract_section_body(content: &str, heading: &str) -> Option<String> {
    let parser = pulldown_cmark::Parser::new_ext(content, pulldown_cmark::Options::all());
    let mut in_target = false;
    let mut body = String::new();
    for event in parser {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading {
                level: pulldown_cmark::HeadingLevel::H2, ..
            }) => { in_target = false; }
            pulldown_cmark::Event::Text(text) if in_target => {
                if text.as_ref().trim() == heading.trim() {
                    in_target = true;
                }
            }
            pulldown_cmark::Event::Text(text) => {
                if text.as_ref().trim() == heading.trim() {
                    in_target = true;
                }
            }
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading { level, .. })
                if level != pulldown_cmark::HeadingLevel::H2 => {
                if in_target { break; }
            }
            _ => {
                if in_target {
                    if let pulldown_cmark::Event::Text(t) = &event {
                        body.push_str(t);
                    }
                }
            }
        }
    }
    if body.is_empty() { None } else { Some(body.trim().to_string()) }
}
```

- [ ] **Step 5: Write `parse_skill_dir`**

Port existing logic — reads `SKILL.md`, parses frontmatter and sections, returns `Option<Skill>`. If `name` is missing from frontmatter, emit a `Diagnostic::Error` instead of silently returning `None` (gap R4-C4 fix).

- [ ] **Step 6: Write `discover_skills`**

Uses `walkdir` to find all `SKILL.md` files under the skills directory. Returns `Vec<Skill>`.

- [ ] **Step 7: Commit**

### Task 1.3: Port YAML config parser

**Files:**
- Create: `crates/loopkit-core/src/parser/yaml.rs`

- [ ] **Step 1: Write `parse_file`**

Generic YAML deserializer, same as old `crate::parser::yaml::parse_file`.

- [ ] **Step 2: Commit**

### Task 1.4: Port config loader

**Files:**
- Create: `crates/loopkit-core/src/config.rs`

- [ ] **Step 1: Write `load_config`**

Looks for `.loopkit.yaml` in root. Falls back to `Config::default()`. Port existing logic.

- [ ] **Step 2: Commit**

### Task 1.5: Expand Config with enforced language fields

**Files:**
- Modify: `crates/loopkit-core/src/types.rs`

- [ ] **Step 1: Add new fields to Config**

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,

    // Enforced language — all have sensible defaults
    #[serde(default = "default_standard_verbs")]
    pub standard_verbs: Vec<String>,
    #[serde(default = "default_halt_reasons")]
    pub halt_reasons: Vec<String>,
    #[serde(default = "default_canonical_loop_sections")]
    pub canonical_loop_sections: Vec<String>,
    #[serde(default = "default_canonical_skill_sections")]
    pub canonical_skill_sections: Vec<String>,
    #[serde(default = "default_state_model_aliases")]
    pub state_model_aliases: Vec<String>,
    #[serde(default = "default_enforced_states")]
    pub enforced_states: Vec<EnforcedState>,
}
```

- [ ] **Step 2: Define `EnforcedState`**

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct EnforcedState {
    pub name: String,
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub description: String,
}
```

- [ ] **Step 3: Write default functions**

```rust
fn default_standard_verbs() -> Vec<String> {
    vec!["trigger","handoff","halt","call","wait","route","escalate","resume","notify","complete"]
        .into_iter().map(String::from).collect()
}

fn default_halt_reasons() -> Vec<String> {
    vec!["stall","ambiguous","human-gate","unsafe","budget"]
        .into_iter().map(String::from).collect()
}

fn default_canonical_loop_sections() -> Vec<String> {
    vec!["Entry Conditions","Loop State Schema","Single Iteration Step",
         "Proof of Progress","State Transition Rule","Halt Conditions","Handoff Target"]
        .into_iter().map(String::from).collect()
}

fn default_canonical_skill_sections() -> Vec<String> {
    vec!["Description","Rules","State Model","Entry Conditions","Halt Conditions"]
        .into_iter().map(String::from).collect()
}

fn default_state_model_aliases() -> Vec<String> {
    vec!["State Model","The Loop","Loop States","States"]
        .into_iter().map(String::from).collect()
}

fn default_enforced_states() -> Vec<EnforcedState> {
    vec![
        EnforcedState { name: "backlog".into(), agent: "coordinator".into(), description: "Picks stories from Linear/Jira/Trello, assigns to PO by priority and dependency".into() },
        EnforcedState { name: "in-analysis".into(), agent: "po-agent".into(), description: "Makes story development-ready".into() },
        EnforcedState { name: "in-dev".into(), agent: "developer".into(), description: "Builds story AC by AC, requests deskcheck per AC".into() },
        EnforcedState { name: "in-deskcheck".into(), agent: "qa-agent".into(), description: "Reviews each AC, returns bug report or approves AC".into() },
        EnforcedState { name: "in-qa".into(), agent: "qa-agent".into(), description: "Full AC check".into() },
        EnforcedState { name: "in-acceptance".into(), agent: "po-agent, ux-agent".into(), description: "Both independently check all ACs".into() },
        EnforcedState { name: "ready-for-deploy".into(), agent: "human".into(), description: "Manual approval gate".into() },
        EnforcedState { name: "done".into(), agent: "".into(), description: "Story moved to done in Linear/Jira/Trello".into() },
        EnforcedState { name: "halted-stall".into(), agent: "".into(), description: "".into() },
        EnforcedState { name: "halted-human-gate".into(), agent: "".into(), description: "".into() },
        EnforcedState { name: "halted-unsafe".into(), agent: "".into(), description: "".into() },
    ]
}
```

- [ ] **Step 4: Update `Default` impl**

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            skills_dir: default_skills_dir(),
            max_iterations: default_max_iterations(),
            standard_verbs: default_standard_verbs(),
            halt_reasons: default_halt_reasons(),
            canonical_loop_sections: default_canonical_loop_sections(),
            canonical_skill_sections: default_canonical_skill_sections(),
            state_model_aliases: default_state_model_aliases(),
            enforced_states: default_enforced_states(),
        }
    }
}
```

- [ ] **Step 5: Commit**

### Task 1.6: State-name validation utility

**Files:**
- Create: `crates/loopkit-core/src/state_name.rs`

- [ ] **Step 1: Write `validate_state_name`**

```rust
/// Returns Ok if name is a valid state name. Rejects:
/// - Empty strings
/// - Names without a hyphen (must be kebab-case)
/// - Names that look like skill names (gerund form: *-ing)
/// - Names with uppercase, spaces, dots, slashes
pub fn validate_state_name(s: &str) -> Result<(), String> {
    if s.is_empty() { return Err("empty state name".into()); }
    if s.len() > 128 { return Err("state name too long (max 128 chars)".into()); }
    if !s.contains('-') { return Err("state name must contain a hyphen (kebab-case)".into()); }
    if s.chars().any(|c| c.is_uppercase() || c.is_whitespace()) {
        return Err("state name must be lowercase with hyphens".into());
    }
    if s.chars().any(|c| c == '.' || c == '/' || c == '\\') {
        return Err("state name must not contain dots or slashes".into());
    }
    Ok(())
}

/// Returns true if the token looks like a state name (not a skill name, URL, or file path).
pub fn is_state_like(s: &str) -> bool {
    s.contains('-') 
        && !s.contains('/') 
        && !s.contains('.') 
        && !s.ends_with("-ing")  // skill names, not states
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
```

- [ ] **Step 2: Add to lib.rs**

```rust
pub mod state_name;
```

- [ ] **Step 3: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_state_names() {
        assert!(validate_state_name("in-dev").is_ok());
        assert!(validate_state_name("ready-for-qa").is_ok());
        assert!(validate_state_name("halted-stall").is_ok());
    }

    #[test]
    fn rejects_skill_names() {
        assert!(validate_state_name("running-tdd-loops").is_err()); // gerund
    }

    #[test]
    fn rejects_no_hyphen() {
        assert!(validate_state_name("done").is_err());
    }

    #[test]
    fn is_state_like_rejects_paths() {
        assert!(!is_state_like("skills/meta/facilitating-inception"));
    }

    #[test]
    fn is_state_like_rejects_urls() {
        assert!(!is_state_like("https://example.com"));
    }

    #[test]
    fn is_state_like_accepts_kebab_states() {
        assert!(is_state_like("in-dev"));
        assert!(is_state_like("ready-for-deskcheck"));
    }
}
```

- [ ] **Step 4: Commit**

---

## Phase 2: loopkit-graph — Loop Validators

### Task 2.1: Port loop-specific types

**Files:**
- Create: `crates/loopkit-graph/src/types.rs`

- [ ] **Step 1: Port `TransitionRule`, `LoopContract`, etc.**

Copy from old `src/types.rs` (the loop-specific parts):
- `LoopContract` with `sections`, `transitions`, `loop_md_path`
- `TransitionRule` with `from`, `to`, `trigger`, `handoff_target`, `handoff_agent`, `halt_reason`, `halt_after`
- `LoopSection` enum (canonical section names plus `Unknown(String)`)
- `Transition` (graph edge) with `from`, `to`, `skill`, `defined_in`

- [ ] **Step 2: Port constants**

```rust
pub const STANDARD_VERBS: &[&str] = &["trigger","handoff","halt","call","wait","route","escalate","resume","notify","complete"];
pub const STANDARD_HALT_REASONS: &[&str] = &["stall","ambiguous","human-gate","unsafe","budget"];
pub const CANONICAL_LOOP_SECTIONS: &[&str] = &["Entry Conditions","Loop State Schema","Single Iteration Step","Proof of Progress","State Transition Rule","Halt Conditions","Handoff Target"];
```

- [ ] **Step 3: Commit**

### Task 2.2: Port loop parsers

**Files:**
- Create: `crates/loopkit-graph/src/parser/mod.rs`
- Create: `crates/loopkit-graph/src/parser/loop_.rs`
- Create: `crates/loopkit-graph/src/parser/handoff.rs`

- [ ] **Step 1: Port `loop_.rs` (LOOP.md section parsing)**

Copy from old `src/parser/loop_.rs`. Replace `crate::parser::skill::parse_sections` with `loopkit_core::parser::skill::parse_sections`.

- [ ] **Step 2: Port `handoff.rs` (transition rule parsing) — FIX gap R4-C1**

Copy from old `src/parser/handoff.rs`. 

Fix: Accept both `→` (Unicode U+2192) and `->` (ASCII) as transition arrow. Update regex:

```rust
// Before: (?P<from>[\w-]+)\s*→\s*(?P<to>[\w-]+)
// After:  (?P<from>[\w-]+)\s*(?:→|->|--->)\s*(?P<to>[\w-]+)

lazy_static! {
    static ref TRANSITION_RE: Regex = Regex::new(
        r"^transition\s+(?P<from>[\w-]+)\s*(?:→|->|--->)\s*(?P<to>[\w-]+)\s*$"
    ).unwrap();
}
```

Also fix backtick-quoted states (gap R4-C3): strip backticks before matching:
```rust
let line = line.trim().replace('`', "");
```

- [ ] **Step 3: Fix `halt after XYZ` parsing (gap R4)**

```rust
// Before: halt_after = caps.name("after").map(|m| m.as_str().parse().unwrap_or(0))
// After:  halt_after = caps.name("after").and_then(|m| m.as_str().parse().ok())
```

- [ ] **Step 4: Fix `has_all_canonical_sections` duplicate bug (gap R4)**

```rust
pub fn has_all_canonical_sections(sections: &[LoopSection]) -> bool {
    let names: std::collections::HashSet<&str> = sections.iter()
        .filter_map(|s| match s {
            LoopSection::Canonical(name) if CANONICAL_LOOP_SECTIONS.contains(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    names.len() == CANONICAL_LOOP_SECTIONS.len()
}
```

- [ ] **Step 5: Commit**

### Task 2.3: Port graph builder

**Files:**
- Create: `crates/loopkit-graph/src/graph.rs`

- [ ] **Step 1: Port graph construction from old `repo.rs`**

Build `Vec<Transition>` from skills' `LoopContract.transitions`. Each transition yields a `Transition` with `from`, `to`, `skill: skill.name.clone()`, `defined_in: skill.loop_md().clone()`.

- [ ] **Step 2: Add auto-detection functions**

```rust
pub fn detect_entry_points(transitions: &[Transition]) -> HashSet<String> { ... }
pub fn detect_terminal_states(transitions: &[Transition]) -> HashSet<String> { ... }
```

- [ ] **Step 3: Commit**

### Task 2.4: Port existing validators

**Files:**
- Create: `crates/loopkit-graph/src/validators/mod.rs`
- Create: `crates/loopkit-graph/src/validators/graph.rs`
- Create: `crates/loopkit-graph/src/validators/state_consistency.rs`
- Create: `crates/loopkit-graph/src/validators/loop_language.rs`
- Create: `crates/loopkit-graph/src/validators/loop_sections.rs`
- Create: `crates/loopkit-graph/src/validators/loop_completeness.rs`
- Create: `crates/loopkit-graph/src/validators/loop_state_files.rs`
- Create: `crates/loopkit-graph/src/validators/cross_references.rs`
- Create: `crates/loopkit-graph/src/validators/constraints.rs`
- Create: `crates/loopkit-graph/src/simulation/mod.rs`

- [ ] **Step 1: Port `graph.rs` — FIX: use config-driven state aliases**

Port old `src/validators/graph.rs`. Use `Config.state_model_aliases` instead of hardcoded `"## State Model"` (gap R2-3 / R3-5 fix).

- [ ] **Step 2: Port `state_consistency.rs` — FIX: use aliases, reverse check**

Port old `src/validators/state_consistency.rs`. Fixes:
- Use `config.state_model_aliases` to find state model sections instead of hardcoded `## State Model` (gap R2-3).
- Add reverse check: every graph node must appear in at least one SKILL.md State Model section (gap R2-R3).
- Use `loopkit_core::state_name::is_state_like` for tighter heuristic (gap R4 false positives).

```rust
// NEW: reverse check
for node in &graph_nodes {
    if !declared_states.contains(node) {
        diagnostics.push(Diagnostic::error(
            "state-undefined-in-prose",
            format!("Graph node '{}' is not declared in any SKILL.md State Model section", node),
            repo.root.join("skills"),
        ));
    }
}
```

- [ ] **Step 3: Port `loop_language.rs` — FIX compound verbs**

Port old `src/validators/loop_language.rs`. Fixes:
- Merge compound verbs: `"Hand off"` → `"handoff"`, `"Cross reference"` → `"cross-reference"`
- Skip temporal conjunctions: `"After"`, `"Before"`, `"Once"`
- Use `config.standard_verbs` and `config.halt_reasons` instead of hardcoded constants

- [ ] **Step 4: Port `loop_sections.rs`**

Port from old. Use `config.canonical_loop_sections`.

- [ ] **Step 5: Port `loop_completeness.rs`**

Port from old. Loop-worthy heuristic: skill has at least one transition.

- [ ] **Step 6: Port `loop_state_files.rs`**

Port from old. Make file paths configurable (gap R3-Req5).

- [ ] **Step 7: Port `cross_references.rs`**

Port from old. Use config for exception lists instead of hardcoded delivery states (gap R3-Req5).

- [ ] **Step 8: Port `constraints.rs`**

Port from old. Pass `config.max_iterations` instead of hardcoded `20` (gap R3-Req5).

- [ ] **Step 9: Port `simulation/mod.rs`**

Port from old. Fix: remove dead `TransitionToUnknownState` check. Accept `max_iterations` param.

- [ ] **Step 10: Write `validators/mod.rs` orchestrator**

```rust
pub fn run_all(config: &loopkit_core::types::Config, skills: &[loopkit_core::types::Skill]) -> Vec<loopkit_core::types::Diagnostic> {
    let transitions = graph::build_transitions(skills);
    let mut diagnostics = Vec::new();

    // Graph validators
    diagnostics.extend(graph::validate(&transitions));
    
    // Simulation (pass config.max_iterations, not hardcoded 20)
    diagnostics.extend(simulation::run_all(&transitions, config.max_iterations));

    // Loop language
    diagnostics.extend(loop_language::validate(skills, config));

    // Loop sections
    diagnostics.extend(loop_sections::validate(skills, config));

    // State consistency (forward + reverse)
    diagnostics.extend(state_consistency::validate(skills, &transitions, config));

    // Enforced states
    diagnostics.extend(enforced_states::validate(&transitions, config));

    // Deskcheck pattern
    diagnostics.extend(deskcheck::validate(&transitions));

    // Bug feedback
    diagnostics.extend(bug_feedback::validate(&transitions));

    // Loop completeness
    diagnostics.extend(loop_completeness::validate(skills));

    // Loop state files
    diagnostics.extend(loop_state_files::validate(config));

    // Cross references
    diagnostics.extend(cross_references::validate(skills, skills));

    // Constraints
    diagnostics.extend(constraints::validate(&transitions, skills));

    diagnostics
}
```

- [ ] **Step 11: Commit**

### Task 2.5: New validator — enforced states

**Files:**
- Create: `crates/loopkit-graph/src/validators/enforced_states.rs`

- [ ] **Step 1: Write `validate` function**

```rust
use loopkit_core::types::{Config, Diagnostic, Severity};
use super::super::types::Transition;

pub fn validate(transitions: &[Transition], config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let graph_states: std::collections::HashSet<&str> = transitions.iter()
        .flat_map(|t| [t.from.as_str(), t.to.as_str()])
        .collect();

    for enforced in &config.enforced_states {
        if !graph_states.contains(enforced.name.as_str()) {
            diagnostics.push(Diagnostic::error(
                "state-enforced-missing",
                format!(
                    "Enforced state '{}' ({}) is not present in any transition graph",
                    enforced.name, enforced.agent
                ),
                std::path::PathBuf::from("skills"),
            ));
        }
    }

    diagnostics
}
```

- [ ] **Step 2: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use loopkit_core::types::Config;

    fn make_transition(from: &str, to: &str) -> Transition {
        Transition {
            from: from.into(), to: to.into(),
            skill: "test-skill".into(),
            defined_in: std::path::PathBuf::from("test/LOOP.md"),
        }
    }

    #[test]
    fn all_enforced_states_present_no_diagnostics() {
        let config = Config::default();
        let transitions = config.enforced_states.iter().flat_map(|s| {
            vec![
                make_transition(&s.name, &s.name), // self-loop
            ]
        }).collect::<Vec<_>>();
        let diags = validate(&transitions, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_enforced_state_emits_error() {
        let config = Config::default();
        let transitions = vec![
            make_transition("in-dev", "in-qa"),
        ];
        let diags = validate(&transitions, &config);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.code == "state-enforced-missing"));
    }
}
```

- [ ] **Step 3: Commit**

### Task 2.6: New validator — deskcheck pattern

**Files:**
- Create: `crates/loopkit-graph/src/validators/deskcheck.rs`

- [ ] **Step 1: Write deskcheck validator**

```rust
pub fn validate(transitions: &[Transition]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let adj = build_adjacency(transitions);
    let has_deskcheck = exists(&adj, "in-deskcheck");

    if !has_deskcheck { return diagnostics; }

    // in-dev must have path to in-deskcheck
    if !has_outbound(&adj, "in-dev", "in-deskcheck") {
        diagnostics.push(Diagnostic::error("state-missing-deskcheck-entry",
            "in-dev must have a transition to in-deskcheck (developer requests QA review per AC)",
            PathBuf::from("skills")));
    }

    // in-deskcheck must have path to in-dev (bug feedback)
    if !has_outbound(&adj, "in-deskcheck", "in-dev") {
        diagnostics.push(Diagnostic::error("state-missing-deskcheck-feedback",
            "in-deskcheck must have a transition back to in-dev (QA returns bug report)",
            PathBuf::from("skills")));
    }

    // in-deskcheck must have path to in-qa (all ACs done)
    if !has_outbound(&adj, "in-deskcheck", "in-qa") {
        diagnostics.push(Diagnostic::error("state-missing-deskcheck-completion",
            "in-deskcheck must have a transition to in-qa (all ACs finalized)",
            PathBuf::from("skills")));
    }

    diagnostics
}
```

- [ ] **Step 2: Commit**

### Task 2.7: New validator — bug feedback

**Files:**
- Create: `crates/loopkit-graph/src/validators/bug_feedback.rs`

- [ ] **Step 1: Write bug feedback validator**

```rust
pub fn validate(transitions: &[Transition]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let adj = build_adjacency(transitions);

    // in-qa must have path back to in-dev (bug found in QA)
    if exists(&adj, "in-qa") && !has_outbound(&adj, "in-qa", "in-dev") {
        diagnostics.push(Diagnostic::error("state-missing-bug-feedback",
            "in-qa must have a transition back to in-dev (bug found, assign back to developer with bug report)",
            PathBuf::from("skills")));
    }

    // in-acceptance must have path back to in-dev (bug in acceptance)
    if exists(&adj, "in-acceptance") && !has_outbound(&adj, "in-acceptance", "in-dev") {
        diagnostics.push(Diagnostic::error("state-missing-bug-feedback",
            "in-acceptance must have a transition back to in-dev (PO/UX finds issues, bug report)",
            PathBuf::from("skills")));
    }

    diagnostics
}
```

- [ ] **Step 2: Commit**

### Task 2.8: Port tests to loopkit-graph

**Files:**
- Create: `crates/loopkit-graph/tests/` (all existing test files)

- [ ] **Step 1: Copy existing tests**

```bash
cp tests/*.rs crates/loopkit-graph/tests/
```

- [ ] **Step 2: Update imports**

Replace `use skill_loop_verifier::*` with `use loopkit_graph::*` and `use loopkit_core::types::*`.

- [ ] **Step 3: Verify tests pass**

```bash
cargo test -p loopkit-graph
```

- [ ] **Step 4: Commit**

---

## Phase 3: loopkit CLI — Best-Practices Checks

### Task 3.1: Frontmatter validator

**Files:**
- Create: `crates/loopkit/src/best_practices/mod.rs`
- Create: `crates/loopkit/src/best_practices/frontmatter.rs`

- [ ] **Step 1: Write frontmatter checks**

```rust
use loopkit_core::types::{Diagnostic, Severity, Skill};
use std::path::PathBuf;

pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    // name checks
    if skill.name.is_empty() {
        diags.push(Diagnostic::error("skill-missing-name", "name field missing from frontmatter".into(), path.clone()));
    } else {
        if skill.name.len() > 64 {
            diags.push(Diagnostic::error("skill-name-too-long", format!("name '{}' exceeds 64 characters ({} chars)", skill.name, skill.name.len()), path.clone()));
        }
        if skill.name.chars().any(|c| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-') {
            diags.push(Diagnostic::error("skill-name-invalid-chars", format!("name '{}' contains invalid characters (only [a-z0-9-] allowed)", skill.name), path.clone()));
        }
        for reserved in &["anthropic", "claude"] {
            if skill.name.contains(reserved) {
                diags.push(Diagnostic::error("skill-name-reserved-word", format!("name '{}' contains reserved word '{}'", skill.name, reserved), path.clone()));
            }
        }
    }

    // description checks
    let desc = &skill.description;
    if desc.is_empty() {
        diags.push(Diagnostic::error("skill-missing-description", "description field missing from frontmatter".into(), path.clone()));
    } else {
        if desc.len() > 1024 {
            diags.push(Diagnostic::error("skill-description-too-long", format!("description exceeds 1024 characters ({} chars)", desc.len()), path.clone()));
        }
        if desc.contains("<") && desc.contains(">") {
            diags.push(Diagnostic::error("skill-description-xml-tag", "description contains XML tags".into(), path.clone()));
        }
        let lower = desc.to_lowercase();
        if lower.starts_with("i ") || lower.starts_with("you ") || lower.starts_with("we ") {
            diags.push(Diagnostic::warning("skill-description-not-third-person", "description appears to use first/second person. Use third person: 'Processes...' not 'I can...'".into(), path.clone()));
        }
    }

    diags
}
```

- [ ] **Step 2: Commit**

### Task 3.2: Naming validator

**Files:**
- Create: `crates/loopkit/src/best_practices/naming.rs`

- [ ] **Step 1: Write naming checks**

Check: gerund form (`-ing`), vague names (`helper`, `utils`, `tools`, `misc`), project-level consistency (count gerund vs noun vs action patterns across all skills, flag if mixed).

- [ ] **Step 2: Commit**

### Task 3.3: Structure validator

**Files:**
- Create: `crates/loopkit/src/best_practices/structure.rs`

- [ ] **Step 1: Write structure checks**

Check: SKILL.md body > 500 lines (Error), deep reference chains (Warning), Windows paths `\` (Error), time-sensitive language (Warning). References >100 lines missing TOC (Warning).

- [ ] **Step 2: Commit**

### Task 3.4: Progressive disclosure validator

**Files:**
- Create: `crates/loopkit/src/best_practices/progressive.rs`

- [ ] **Step 1: Write progressive disclosure checks**

Check: SKILL.md > 200 lines but no reference files (Warning), orphan reference files not linked from SKILL.md (Warning).

- [ ] **Step 2: Commit**

### Task 3.5: Terminology validator

**Files:**
- Create: `crates/loopkit/src/best_practices/terminology.rs`

- [ ] **Step 1: Write terminology consistency checks**

Check: common synonym pairs (endpoint/route/URL, field/element/control, extract/pull/get/retrieve). If multiple synonyms appear in the same skill, flag.

- [ ] **Step 2: Commit**

### Task 3.6: Workflow validator

**Files:**
- Create: `crates/loopkit/src/best_practices/workflow.rs`

- [ ] **Step 1: Write workflow quality checks**

Check: missing checklist for multi-step procedures, missing feedback loop (validate→fix→repeat), missing conditional structure for branching logic.

- [ ] **Step 2: Commit**

### Task 3.7: Anti-patterns validator

**Files:**
- Create: `crates/loopkit/src/best_practices/antipatterns.rs`

- [ ] **Step 1: Write anti-pattern checks**

Check: >3 equivalent options for same task (Warning), undocumented magic numbers (Warning), bare `raise` without error handling (Warning).

- [ ] **Step 2: Commit**

### Task 3.8: Best-practices orchestrator

**Files:**
- Modify: `crates/loopkit/src/best_practices/mod.rs`

- [ ] **Step 1: Write `check_all`**

```rust
pub fn check_all(skills: &[loopkit_core::types::Skill]) -> Vec<loopkit_core::types::Diagnostic> {
    let mut diagnostics = Vec::new();
    for skill in skills {
        diagnostics.extend(frontmatter::check(skill));
        diagnostics.extend(naming::check(skill));
        diagnostics.extend(structure::check(skill));
        diagnostics.extend(progressive::check(skill));
        diagnostics.extend(terminology::check(skill));
        diagnostics.extend(workflow::check(skill));
        diagnostics.extend(antipatterns::check(skill));
    }
    // Cross-skill checks (naming consistency across project)
    diagnostics.extend(naming::check_consistency(skills));
    diagnostics
}
```

- [ ] **Step 2: Commit**

---

## Phase 4: Integration & CLI

### Task 4.1: Wire CLI main

**Files:**
- Modify: `crates/loopkit/src/main.rs`

- [ ] **Step 1: Write `main.rs` using clap**

```rust
use clap::Parser;
use loopkit_core::{config::load_config, diagnostic::{format_diagnostics, diagnostics_json}};
use loopkit_core::parser::skill::discover_skills;
use loopkit_core::types::Severity;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "loopkit", about = "Prove your agent skill loops are correct")]
struct Cli {
    /// Path to skills directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output JSON instead of text
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let config = load_config(&cli.path);
    let skills_dir = cli.path.join(&config.skills_dir);

    let skills = match discover_skills(&skills_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to discover skills: {}", e);
            std::process::exit(2);
        }
    };

    let mut diagnostics = Vec::new();

    // Loop/graph validation
    diagnostics.extend(loopkit_graph::validators::run_all(&config, &skills));

    // Best-practices checks
    diagnostics.extend(loopkit::best_practices::check_all(&skills));

    let error_count = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();

    if cli.json {
        println!("{}", diagnostics_json(&diagnostics, skills.len()));
    } else {
        println!("{}", format_diagnostics(&diagnostics));
        println!("\n{} skills checked. {} error(s), {} warning(s).",
            skills.len(),
            error_count,
            diagnostics.len() - error_count,
        );
    }

    if error_count > 0 {
        std::process::exit(1);
    }
}
```

- [ ] **Step 2: Add output formatting to CLI crate**

Move `diagnostic.rs` formatting functions from core to CLI crate, or keep them in core (they only depend on `Diagnostic` types). Use existing `format_diagnostics` and `diagnostics_json`.

- [ ] **Step 3: Verify binary builds and runs**

```bash
cargo build -p loopkit
cargo run -p loopkit -- /Users/canavar/projects/forge
```

Expected: output showing diagnostics for forge skills.

- [ ] **Step 4: Commit**

### Task 4.2: Run against forge, fix issues

- [ ] **Step 1: Run loopkit against forge**

```bash
cargo run -p loopkit -- --json /Users/canavar/projects/forge > /tmp/loopkit-forge.json
```

- [ ] **Step 2: Verify zero Errors**

Count errors in JSON output. Expected: 0 errors. Warnings are acceptable.

- [ ] **Step 3: Fix any unexpected errors found**

If forge skills violate any new enforced rules, either fix the skills or adjust the defaults.

- [ ] **Step 4: Commit**

---

## Phase 5: Polish

### Task 5.1: Update .loopkit.yaml with expanded config

**Files:**
- Create: `.loopkit.yaml`

- [ ] **Step 1: Write default config**

```yaml
skills_dir: skills/
max_iterations: 20
```

- [ ] **Step 2: Commit**

### Task 5.2: Build release binary

- [ ] **Step 1: Build release**

```bash
cargo build --release
```

- [ ] **Step 2: Verify binary size and speed**

```bash
ls -lh target/release/loopkit
time target/release/loopkit /Users/canavar/projects/forge
```

- [ ] **Step 3: Commit**

---

## Self-Review Checklist

After all phases complete, verify:

1. **Spec coverage:** Every requirement in the spec maps to at least one task above.
2. **No placeholders:** No "TBD", "TODO", "add error handling" in task steps.
3. **Type consistency:** `Config.enforced_states` is `Vec<EnforcedState>` everywhere — not `Vec<String>`.
4. **All 15 gap fixes addressed:** Each gap from spec Section 7 has a corresponding fix task.
5. **Zero hardcoded variables:** `Config` is the single source of truth — no literal `"## State Model"`, `20`, or `"skills/"` in validators.
6. **Workspace builds clean:** `cargo build` from root compiles all three crates.
7. **Tests pass:** `cargo test` passes for all crates.

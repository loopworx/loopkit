use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// ── Loop language: standard halt reasons ──────────────────────────────

pub const STANDARD_HALT_REASONS: &[&str] = &[
    "stall",
    "ambiguous",
    "human-gate",
    "unsafe",
    "budget",
];

// ── Loop language: standard action verbs ─────────────────────────────

pub const STANDARD_VERBS: &[&str] = &[
    "verify",
    "write",
    "commit",
    "handoff",
    "halt",
    "gate",
    "pull",
    "read",
    "run",
    "check",
    "confirm",
    "create",
    "update",
    "define",
];

// ── Loop language: canonical LOOP.md section headings ────────────────

pub const CANONICAL_LOOP_SECTIONS: &[&str] = &[
    "Entry Conditions",
    "Loop State Schema",
    "Single Iteration Step",
    "Proof of Progress",
    "State Transition Rule",
    "Halt Conditions",
    "Handoff Target",
];

pub const CANONICAL_LOOP_SECTION_COUNT: usize = 7;

// ── Loop language: canonical SKILL.md section headings ───────────────

pub const CANONICAL_SKILL_SECTIONS: &[&str] = &[
    "Description",
    "Rules",
];

/// Alternative names for the State Model section.
pub const STATE_MODEL_ALIASES: &[&str] = &[
    "State Model",
    "The Loop",
    "Loop States",
    "Loop State",
    "States",
];

// ── L1 rigid extra required sections ──────────────────────────────────

pub const L1_RIGID_EXTRA_SECTIONS: &[&str] = &[
    "Entry Conditions",
    "Halt Conditions",
];

// ── Transition rule syntax constants ──────────────────────────────────

/// Regex pattern for a transition directive line.
/// Example: `transition in-dev → ready-for-deskcheck`
pub const TRANSITION_LINE_PATTERN: &str =
    r"^transition\s+(?P<from>[\w-]+)\s*→\s*(?P<to>[\w-]+)\s*$";

/// Possible keywords in a transition block.
pub const TRANSITION_KEYWORDS: &[&str] = &[
    "trigger",
    "handoff",
    "halt",
];

// ── Core types (all strings, no hardcoded enums) ─────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct State {
    pub name: String,
    /// Which skills define this state (via transition rules or state model).
    pub defined_in: Vec<String>,
    pub is_entry: bool,
    pub is_terminal: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Skill {
    pub name: String,
    pub category: String,
    pub path: PathBuf,
    pub level: String,
    pub owner: Vec<String>,
    pub sections: Vec<String>,
    pub states: Vec<State>,
    pub has_loop_md: bool,
    pub has_handoffs_md: bool,
    /// Parsed transition rules from LOOP.md (or HANDOFFS.md for backwards compat).
    pub transitions: Vec<TransitionRule>,
}

impl Skill {
    pub fn dir(&self) -> &std::path::Path {
        self.path.as_path()
    }

    pub fn skill_md(&self) -> PathBuf {
        self.path.join("SKILL.md")
    }

    pub fn handoffs_md(&self) -> PathBuf {
        self.path.join("HANDOFFS.md")
    }

    pub fn loop_md(&self) -> PathBuf {
        self.path.join("LOOP.md")
    }
}

/// A parsed transition rule from LOOP.md
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransitionRule {
    pub from: String,
    pub to: String,
    pub trigger: Option<String>,
    pub handoff_skill: Option<String>,
    pub handoff_agent: Option<String>,
    pub halt_reason: Option<String>,
    pub halt_after: Option<u32>,
    pub defined_in: String, // skill name
}

/// Transition edge in the handoff graph (deduplicated from TransitionRule).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub trigger: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HandoffGraph {
    pub nodes: Vec<State>,
    pub edges: Vec<Transition>,
    pub entry_points: Vec<State>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Repo {
    pub root: PathBuf,
    pub skills: Vec<Skill>,
    pub handoff_graph: HandoffGraph,
}

// ── Loop contract ────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoopContract {
    pub skill: String,
    pub sections: Vec<LoopSection>,
    pub section_order_valid: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum LoopSection {
    EntryConditions(String),
    LoopStateSchema(String),
    SingleIterationStep(String),
    ProofOfProgress(String),
    StateTransitionRule(String),
    HaltConditions(String),
    HandoffTarget(String),
    Unknown(String),
}

// ── Loop language violations ─────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoopLanguageViolation {
    pub skill: String,
    pub code: String,
    pub message: String,
    pub line: usize,
}

pub fn is_standard_verb(word: &str) -> bool {
    STANDARD_VERBS.contains(&word.to_lowercase().as_str())
}

pub fn is_standard_halt_reason(reason: &str) -> bool {
    STANDARD_HALT_REASONS.contains(&reason.to_lowercase().as_str())
}

pub fn is_canonical_loop_section(heading: &str) -> bool {
    CANONICAL_LOOP_SECTIONS.iter().any(|s| *s == heading)
}

pub fn is_state_model_alias(heading: &str) -> bool {
    STATE_MODEL_ALIASES.iter().any(|a| *a == heading)
}

pub fn is_canonical_skill_section(heading: &str) -> bool {
    CANONICAL_SKILL_SECTIONS.contains(&heading) || is_state_model_alias(heading)
}

/// Validate a halt reason string. Returns None if valid, Some(error) if not.
pub fn validate_halt_reason(reason: &str) -> Option<String> {
    if is_standard_halt_reason(reason) {
        None
    } else {
        Some(format!(
            "unknown halt reason '{}'. Must be one of: {}",
            reason,
            STANDARD_HALT_REASONS.join(", ")
        ))
    }
}

/// Validate an action verb. Returns None if valid, Some(error) if not.
pub fn validate_verb(verb: &str) -> Option<String> {
    if is_standard_verb(verb) {
        None
    } else {
        Some(format!(
            "unknown verb '{}'. Must be one of: {}",
            verb,
            STANDARD_VERBS.join(", ")
        ))
    }
}

// ── Configuration ────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
}

fn default_skills_dir() -> String {
    "skills/".to_string()
}

fn default_max_iterations() -> u32 {
    20
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skills_dir: default_skills_dir(),
            max_iterations: default_max_iterations(),
        }
    }
}

// ── Diagnostics ──────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileLocation {
    pub path: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub location: FileLocation,
    pub help: String,
}

// ── Helper types for graph auto-discovery ────────────────────────────

/// Build adjacency from transitions (for graph analysis).
pub fn build_adjacency(transitions: &[Transition]) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for t in transitions {
        if t.from != t.to {
            adj.entry(t.from.clone()).or_default().push(t.to.clone());
        }
    }
    for v in adj.values_mut() {
        v.sort();
        v.dedup();
    }
    adj
}

/// Auto-detect entry points: states with zero inbound transitions.
pub fn detect_entry_points(transitions: &[Transition]) -> HashSet<String> {
    let all_states: HashSet<String> = transitions
        .iter()
        .flat_map(|t| [t.from.clone(), t.to.clone()])
        .collect();
    let has_inbound: HashSet<String> = transitions
        .iter()
        .map(|t| t.to.clone())
        .collect();
    all_states
        .into_iter()
        .filter(|s| !has_inbound.contains(s))
        .collect()
}

/// Auto-detect terminal states: states with zero outbound transitions.
pub fn detect_terminal_states(transitions: &[Transition]) -> HashSet<String> {
    let all_states: HashSet<String> = transitions
        .iter()
        .flat_map(|t| [t.from.clone(), t.to.clone()])
        .collect();
    let has_outbound: HashSet<String> = transitions
        .iter()
        .map(|t| t.from.clone())
        .collect();
    all_states
        .into_iter()
        .filter(|s| !has_outbound.contains(s))
        .collect()
}

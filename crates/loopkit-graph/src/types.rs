/// Standard verbs allowed in transition rules.
pub const STANDARD_VERBS: &[&str] = &[
    "trigger", "handoff", "halt", "call", "wait", "route", "escalate", "resume", "notify", "complete"
];

/// Standard halt reasons.
pub const STANDARD_HALT_REASONS: &[&str] = &[
    "stall", "ambiguous", "human-gate", "unsafe", "budget"
];

/// Canonical LOOP.md section names (in order).
pub const CANONICAL_LOOP_SECTIONS: &[&str] = &[
    "Entry Conditions", "Loop State Schema", "Single Iteration Step",
    "Proof of Progress", "State Transition Rule", "Halt Conditions", "Handoff Target"
];

/// A transition rule parsed from a LOOP.md file.
#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub from: String,
    pub to: String,
    pub trigger: Option<String>,
    pub handoff_target: Option<String>,
    pub handoff_agent: Option<String>,
    pub halt_reason: Option<String>,
    pub halt_after: Option<u32>,
    pub defined_in: String,
}

/// A resolved transition edge in the handoff graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub skill: String,
    pub defined_in: std::path::PathBuf,
}

/// Sections of a LOOP.md file.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl LoopSection {
    pub fn name(&self) -> &str {
        match self {
            LoopSection::EntryConditions(_) => "Entry Conditions",
            LoopSection::LoopStateSchema(_) => "Loop State Schema",
            LoopSection::SingleIterationStep(_) => "Single Iteration Step",
            LoopSection::ProofOfProgress(_) => "Proof of Progress",
            LoopSection::StateTransitionRule(_) => "State Transition Rule",
            LoopSection::HaltConditions(_) => "Halt Conditions",
            LoopSection::HandoffTarget(_) => "Handoff Target",
            LoopSection::Unknown(name) => name,
        }
    }

    pub fn body(&self) -> &str {
        match self {
            LoopSection::EntryConditions(b) => b,
            LoopSection::LoopStateSchema(b) => b,
            LoopSection::SingleIterationStep(b) => b,
            LoopSection::ProofOfProgress(b) => b,
            LoopSection::StateTransitionRule(b) => b,
            LoopSection::HaltConditions(b) => b,
            LoopSection::HandoffTarget(b) => b,
            LoopSection::Unknown(_) => "",
        }
    }
}

/// Parsed representation of a LOOP.md file.
#[derive(Debug, Clone)]
pub struct LoopContract {
    pub skill: String,
    pub sections: Vec<LoopSection>,
    pub section_order_valid: bool,
    pub transitions: Vec<TransitionRule>,
    pub loop_md_path: std::path::PathBuf,
}

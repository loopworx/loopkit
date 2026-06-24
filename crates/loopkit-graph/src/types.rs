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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_section_name() {
        assert_eq!(LoopSection::EntryConditions("body".into()).name(), "Entry Conditions");
        assert_eq!(LoopSection::LoopStateSchema("body".into()).name(), "Loop State Schema");
        assert_eq!(LoopSection::SingleIterationStep("body".into()).name(), "Single Iteration Step");
        assert_eq!(LoopSection::ProofOfProgress("body".into()).name(), "Proof of Progress");
        assert_eq!(LoopSection::StateTransitionRule("body".into()).name(), "State Transition Rule");
        assert_eq!(LoopSection::HaltConditions("body".into()).name(), "Halt Conditions");
        assert_eq!(LoopSection::HandoffTarget("body".into()).name(), "Handoff Target");
        assert_eq!(LoopSection::Unknown("Custom Section".into()).name(), "Custom Section");
    }

    #[test]
    fn loop_section_body() {
        assert_eq!(LoopSection::EntryConditions("the body".into()).body(), "the body");
        assert_eq!(LoopSection::LoopStateSchema("schema body".into()).body(), "schema body");
        assert_eq!(LoopSection::SingleIterationStep("step body".into()).body(), "step body");
        assert_eq!(LoopSection::ProofOfProgress("proof body".into()).body(), "proof body");
        assert_eq!(LoopSection::StateTransitionRule("rule body".into()).body(), "rule body");
        assert_eq!(LoopSection::HaltConditions("halt body".into()).body(), "halt body");
        assert_eq!(LoopSection::HandoffTarget("handoff body".into()).body(), "handoff body");
        assert_eq!(LoopSection::Unknown("whatever".into()).body(), "");
    }

    #[test]
    fn standard_verbs_has_expected_values() {
        assert_eq!(STANDARD_VERBS.len(), 10);
        assert!(STANDARD_VERBS.contains(&"trigger"));
        assert!(STANDARD_VERBS.contains(&"handoff"));
        assert!(STANDARD_VERBS.contains(&"halt"));
        assert!(STANDARD_VERBS.contains(&"call"));
        assert!(STANDARD_VERBS.contains(&"wait"));
        assert!(STANDARD_VERBS.contains(&"route"));
        assert!(STANDARD_VERBS.contains(&"escalate"));
        assert!(STANDARD_VERBS.contains(&"resume"));
        assert!(STANDARD_VERBS.contains(&"notify"));
        assert!(STANDARD_VERBS.contains(&"complete"));
    }

    #[test]
    fn standard_halt_reasons_has_expected_values() {
        assert_eq!(STANDARD_HALT_REASONS.len(), 5);
        assert!(STANDARD_HALT_REASONS.contains(&"stall"));
        assert!(STANDARD_HALT_REASONS.contains(&"ambiguous"));
        assert!(STANDARD_HALT_REASONS.contains(&"human-gate"));
        assert!(STANDARD_HALT_REASONS.contains(&"unsafe"));
        assert!(STANDARD_HALT_REASONS.contains(&"budget"));
    }

    #[test]
    fn canonical_loop_sections_has_all_seven() {
        assert_eq!(CANONICAL_LOOP_SECTIONS.len(), 7);
        assert_eq!(CANONICAL_LOOP_SECTIONS[0], "Entry Conditions");
        assert_eq!(CANONICAL_LOOP_SECTIONS[1], "Loop State Schema");
        assert_eq!(CANONICAL_LOOP_SECTIONS[2], "Single Iteration Step");
        assert_eq!(CANONICAL_LOOP_SECTIONS[3], "Proof of Progress");
        assert_eq!(CANONICAL_LOOP_SECTIONS[4], "State Transition Rule");
        assert_eq!(CANONICAL_LOOP_SECTIONS[5], "Halt Conditions");
        assert_eq!(CANONICAL_LOOP_SECTIONS[6], "Handoff Target");
    }
}

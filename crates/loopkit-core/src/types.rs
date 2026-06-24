use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileLocation {
    pub path: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl FileLocation {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            line: None,
            column: None,
        }
    }

    pub fn at(path: PathBuf, line: u32, column: u32) -> Self {
        Self {
            path,
            line: Some(line),
            column: Some(column),
        }
    }

    pub fn at_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub location: FileLocation,
    pub help: String,
}

impl Diagnostic {
    pub fn error(code: &str, message: String, path: PathBuf) -> Self {
        Self {
            severity: Severity::Error,
            code: code.to_string(),
            message,
            location: FileLocation::new(path),
            help: String::new(),
        }
    }

    pub fn warning(code: &str, message: String, path: PathBuf) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.to_string(),
            message,
            location: FileLocation::new(path),
            help: String::new(),
        }
    }

    pub fn info(code: &str, message: String, path: PathBuf) -> Self {
        Self {
            severity: Severity::Info,
            code: code.to_string(),
            message,
            location: FileLocation::new(path),
            help: String::new(),
        }
    }

    pub fn at_line(mut self, line: u32) -> Self {
        self.location.line = Some(line);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = help;
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Section {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Skill {
    pub name: String,
    pub level: String,
    pub owner: Vec<String>,
    pub description: String,
    pub category: String,
    pub path: PathBuf,
    pub skill_md: PathBuf,
    pub sections: Vec<Section>,
    pub states: Vec<String>,
}

impl Skill {
    pub fn loop_md(&self) -> PathBuf {
        self.path.join("LOOP.md")
    }

    pub fn handoffs_md(&self) -> PathBuf {
        self.path.join("HANDOFFS.md")
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct EnforcedState {
    pub name: String,
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,

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

fn default_skills_dir() -> String { "skills/".to_string() }
fn default_max_iterations() -> u32 { 20 }

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
    vec![]
}

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

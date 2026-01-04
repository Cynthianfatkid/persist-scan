use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub os: String, // "linux" or "windows" (human-readable)
    pub severity: Severity,
    pub confidence: Confidence,
    pub tags: Vec<String>,
    pub rationale: String,
    pub check: Check,
    pub r#match: Match,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub kind: String, // maps to Artifact.kind
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Match {
    #[serde(default)]
    pub any_path_contains: Vec<String>,
    #[serde(default)]
    pub any_path_prefix: Vec<String>,
    #[serde(default)]
    pub any_command_contains: Vec<String>,
    #[serde(default)]
    pub regex_command: Vec<String>,
    #[serde(default)]
    pub regex_path: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn points(self) -> i32 {
        match self {
            Severity::Low => 5,
            Severity::Medium => 15,
            Severity::High => 30,
        }
    }
}

impl Confidence {
    pub fn multiplier(self) -> f32 {
        match self {
            Confidence::Low => 0.7,
            Confidence::Medium => 1.0,
            Confidence::High => 1.2,
        }
    }
}

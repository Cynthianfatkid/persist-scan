use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

pub mod matcher;
pub mod report;
pub mod rule;

use matcher::rule_matches;
use report::{Finding, Report};
use rule::Rule;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetOs {
    Linux,
    Windows,
}

pub fn detect_os(arg: &str) -> Result<TargetOs> {
    match arg.to_lowercase().as_str() {
        "auto" => {
            if cfg!(windows) {
                Ok(TargetOs::Windows)
            } else if cfg!(unix) {
                Ok(TargetOs::Linux)
            } else {
                Err(anyhow!("Unsupported host OS for auto-detect"))
            }
        }
        "linux" => Ok(TargetOs::Linux),
        "windows" => Ok(TargetOs::Windows),
        _ => Err(anyhow!("Invalid --os value: {} (use auto|linux|windows)", arg)),
    }
}

pub fn load_rules(rules_dir: &str, os: TargetOs) -> Result<Vec<Rule>> {
    let subdir = match os {
        TargetOs::Linux => "linux",
        TargetOs::Windows => "windows",
    };
    let dir = std::path::Path::new(rules_dir).join(subdir);
    let mut out = vec![];

    if !dir.exists() {
        return Err(anyhow!("Rules directory not found: {}", dir.display()));
    }

    for entry in walkdir::WalkDir::new(&dir).max_depth(2) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "yml" && ext != "yaml" {
            continue;
        }

        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed reading rule file {}", path.display()))?;
        let rule: Rule = serde_yaml::from_slice(&bytes)
            .with_context(|| format!("Failed parsing YAML {}", path.display()))?;
        out.push(rule);
    }

    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// A generic “artifact” from collectors; rules match against these.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Artifact {
    pub kind: String,            // e.g. "windows_run_key", "linux_autostart"
    pub source: String,          // e.g. registry path, filename, "crontab -l"
    pub name: String,            // entry name or unit name
    pub command: Option<String>, // command/exec line if available
    pub path: Option<String>,    // extracted executable path if we can parse it
    pub raw: Option<String>,     // raw line if useful
}

pub fn run(rules: &[Rule], artifacts: &[Artifact]) -> Result<Vec<Finding>> {
    let mut findings = vec![];
    for rule in rules {
        for art in artifacts {
            if rule_matches(rule, art)? {
                findings.push(Finding::from_match(rule, art));
            }
        }
    }
    Ok(findings)
}

pub fn build_report(os: TargetOs, artifacts: Vec<Artifact>, findings: Vec<Finding>) -> Report {
    Report::new(os, artifacts, findings)
}

/// Baseline snapshot: store only artifacts (rules can change independently)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSnapshot {
    pub os: TargetOs,
    pub collected_at_utc: chrono::DateTime<chrono::Utc>,
    pub artifacts: Vec<Artifact>,
}

impl ArtifactSnapshot {
    pub fn from_artifacts(os: TargetOs, mut artifacts: Vec<Artifact>) -> Self {
        artifacts.sort_by(|a, b| format_key(a).cmp(&format_key(b)));
        artifacts.dedup_by(|a, b| format_key(a) == format_key(b));
        Self {
            os,
            collected_at_utc: chrono::Utc::now(),
            artifacts,
        }
    }
}

fn format_key(a: &Artifact) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        a.kind,
        a.source,
        a.name,
        a.command.as_deref().unwrap_or(""),
        a.path.as_deref().unwrap_or(""),
        a.raw.as_deref().unwrap_or("")
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDelta {
    pub added: Vec<Artifact>,
    pub removed: Vec<Artifact>,
}

pub fn diff_snapshots(base: &ArtifactSnapshot, current: &ArtifactSnapshot) -> SnapshotDelta {
    use std::collections::HashSet;
    let b: HashSet<String> = base.artifacts.iter().map(format_key).collect();
    let c: HashSet<String> = current.artifacts.iter().map(format_key).collect();

    let mut added = vec![];
    let mut removed = vec![];

    for a in &current.artifacts {
        if !b.contains(&format_key(a)) {
            added.push(a.clone());
        }
    }
    for a in &base.artifacts {
        if !c.contains(&format_key(a)) {
            removed.push(a.clone());
        }
    }

    SnapshotDelta { added, removed }
}

/// Convert delta into artifacts for scanning: scan "added" only (and optionally "removed")
pub fn artifacts_from_delta(delta: &SnapshotDelta, include_removed: bool) -> Vec<Artifact> {
    let mut out = vec![];
    out.extend(delta.added.iter().cloned());
    if include_removed {
        for mut a in delta.removed.clone() {
            a.source = format!("(REMOVED) {}", a.source);
            a.name = format!("(REMOVED) {}", a.name);
            out.push(a);
        }
    }
    out
}

impl Report {
    pub fn with_delta(mut self, delta: SnapshotDelta) -> Self {
        self.delta = Some(delta);
        self
    }
}

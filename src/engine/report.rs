use serde::{Deserialize, Serialize};

use super::{Artifact, TargetOs, SnapshotDelta};
use super::rule::{Confidence, Severity, Rule};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub title: String,
    pub severity: Severity,
    pub confidence: Confidence,
    pub tags: Vec<String>,
    pub rationale: String,

    pub artifact_kind: String,
    pub source: String,
    pub name: String,
    pub command: Option<String>,
    pub path: Option<String>,
}

impl Finding {
    pub fn from_match(rule: &Rule, art: &Artifact) -> Self {
        Self {
            rule_id: rule.id.clone(),
            title: rule.title.clone(),
            severity: rule.severity,
            confidence: rule.confidence,
            tags: rule.tags.clone(),
            rationale: rule.rationale.clone(),
            artifact_kind: art.kind.clone(),
            source: art.source.clone(),
            name: art.name.clone(),
            command: art.command.clone(),
            path: art.path.clone(),
        }
    }

    pub fn score(&self) -> f32 {
        self.severity.points() as f32 * self.confidence.multiplier()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub os: TargetOs,
    pub generated_at_utc: chrono::DateTime<chrono::Utc>,
    pub risk_score_0_100: i32,
    pub counts: Counts,
    pub findings: Vec<Finding>,
    pub artifacts_scanned: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<SnapshotDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Counts {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

impl Report {
    pub fn new(os: TargetOs, artifacts: Vec<Artifact>, mut findings: Vec<Finding>) -> Self {
        findings.sort_by(|a, b| {
            let sa = sev_rank(a.severity);
            let sb = sev_rank(b.severity);
            sb.cmp(&sa).then(a.rule_id.cmp(&b.rule_id))
        });

        let mut counts = Counts::default();
        for f in &findings {
            match f.severity {
                Severity::High => counts.high += 1,
                Severity::Medium => counts.medium += 1,
                Severity::Low => counts.low += 1,
            }
        }

        let total: f32 = findings.iter().map(|f| f.score()).sum();
        let mut score = total.round() as i32;
        if score > 100 { score = 100; }
        if score < 0 { score = 0; }

        Self {
            os,
            generated_at_utc: chrono::Utc::now(),
            risk_score_0_100: score,
            counts,
            artifacts_scanned: artifacts.len(),
            findings,
            delta: None,
        }
    }

    pub fn to_human_readable(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "Risk score: {} / 100\nFindings: {} high, {} medium, {} low\nArtifacts scanned: {}\n",
            self.risk_score_0_100, self.counts.high, self.counts.medium, self.counts.low, self.artifacts_scanned
        ));

        if let Some(d) = &self.delta {
            s.push_str(&format!(
                "Delta: +{} added, -{} removed\n",
                d.added.len(),
                d.removed.len()
            ));
        }

        for f in &self.findings {
            s.push_str("\n");
            s.push_str(&format!(
                "[{:?}] {} ({})\n  Source: {}\n  Name: {}\n",
                f.severity, f.rule_id, f.title, f.source, f.name
            ));
            if let Some(cmd) = &f.command {
                s.push_str(&format!("  Command: {}\n", cmd));
            }
            if let Some(path) = &f.path {
                s.push_str(&format!("  Path: {}\n", path));
            }
            s.push_str(&format!(
                "  Confidence: {:?}   Tags: {}\n  Why: {}\n",
                f.confidence,
                f.tags.join(", "),
                f.rationale
            ));
        }

        s
    }
}

fn sev_rank(s: Severity) -> i32 {
    match s {
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
    }
}

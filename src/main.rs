use anyhow::{Context, Result};

mod cli;
mod collectors;
mod engine;
mod util;

use clap::Parser;
use cli::{Args, Command};

fn main() -> Result<()> {
    let args = Args::parse();

    let os = engine::detect_os(&args.os)?;
    let rules = engine::load_rules(&args.rules_dir, os)
        .with_context(|| format!("Failed loading rules from {}", args.rules_dir))?;

    match args.cmd {
        Command::Scan => {
            let artifacts = collectors::collect(os)?;
            let findings = engine::run(&rules, &artifacts)?;
            let report = engine::build_report(os, artifacts, findings);

            if args.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", report.to_human_readable());
            }
        }
        Command::Baseline { out } => {
            let artifacts = collectors::collect(os)?;
            let snapshot = engine::ArtifactSnapshot::from_artifacts(os, artifacts);
            std::fs::write(&out, serde_json::to_vec_pretty(&snapshot)?)?;
            println!("Wrote baseline snapshot to {}", out);
        }
        Command::Diff { baseline, show_removed, json } => {
            let base_bytes = std::fs::read(&baseline)
                .with_context(|| format!("Could not read baseline file: {}", baseline))?;
            let base: engine::ArtifactSnapshot = serde_json::from_slice(&base_bytes)
                .with_context(|| "Baseline JSON format invalid")?;

            let current_artifacts = collectors::collect(os)?;
            let current_snapshot = engine::ArtifactSnapshot::from_artifacts(os, current_artifacts.clone());

            let delta = engine::diff_snapshots(&base, &current_snapshot);
            let delta_artifacts = engine::artifacts_from_delta(&delta, show_removed);

            let findings = engine::run(&rules, &delta_artifacts)?;
            let report = engine::build_report(os, delta_artifacts, findings).with_delta(delta);

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", report.to_human_readable());
            }
        }
    }

    Ok(())
}

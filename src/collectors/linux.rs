use anyhow::{Context, Result};
use std::process::Command;
use walkdir::WalkDir;

use crate::engine::Artifact;
use crate::util::paths::{extract_executable_path_guess, expand_tilde};

pub fn collect_linux() -> Result<Vec<Artifact>> {
    let mut out = vec![];
    out.extend(collect_autostart()?);
    out.extend(collect_user_crontab()?);
    Ok(out)
}

fn collect_autostart() -> Result<Vec<Artifact>> {
    let mut out = vec![];
    let dir = expand_tilde("~/.config/autostart");
    if !dir.exists() {
        return Ok(out);
    }

    for entry in WalkDir::new(&dir).max_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed reading {}", path.display()))?;

        let exec_line = content
            .lines()
            .find(|l| l.trim_start().starts_with("Exec="))
            .map(|l| l.trim().to_string());

        let cmd = exec_line.as_ref().map(|l| l.trim_start_matches("Exec=").to_string());
        let path_guess = cmd.as_deref().and_then(extract_executable_path_guess);

        out.push(Artifact {
            kind: "linux_autostart".to_string(),
            source: path.display().to_string(),
            name: path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string(),
            command: cmd,
            path: path_guess,
            raw: None,
        });
    }

    Ok(out)
}

fn collect_user_crontab() -> Result<Vec<Artifact>> {
    let mut out = vec![];

    let output = Command::new("crontab").arg("-l").output();
    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(out),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        return Ok(out);
    }

    for (idx, line) in stdout.lines().enumerate() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }

        let cmd = guess_cron_command(l);
        let path_guess = cmd.as_deref().and_then(extract_executable_path_guess);

        out.push(Artifact {
            kind: "linux_user_crontab".to_string(),
            source: "crontab -l".to_string(),
            name: format!("line:{}", idx + 1),
            command: cmd,
            path: path_guess,
            raw: Some(l.to_string()),
        });
    }

    Ok(out)
}

fn guess_cron_command(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return Some(line.to_string());
    }
    Some(parts[5..].join(" "))
}

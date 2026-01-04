use anyhow::Result;
use regex::Regex;

use super::Artifact;
use super::rule::Rule;

pub fn rule_matches(rule: &Rule, art: &Artifact) -> Result<bool> {
    if rule.check.kind != art.kind {
        return Ok(false);
    }

    let m = &rule.r#match;
    let has_criteria = !m.any_path_contains.is_empty()
        || !m.any_path_prefix.is_empty()
        || !m.any_command_contains.is_empty()
        || !m.regex_command.is_empty()
        || !m.regex_path.is_empty();

    if !has_criteria {
        return Ok(true);
    }

    let path = art.path.as_deref().unwrap_or("");
    let cmd = art.command.as_deref().unwrap_or("");

    for needle in &m.any_path_contains {
        if !needle.is_empty() && path.to_lowercase().contains(&needle.to_lowercase()) {
            return Ok(true);
        }
    }
    for pref in &m.any_path_prefix {
        if !pref.is_empty() && path.starts_with(pref) {
            return Ok(true);
        }
    }
    for needle in &m.any_command_contains {
        if !needle.is_empty() && cmd.to_lowercase().contains(&needle.to_lowercase()) {
            return Ok(true);
        }
    }
    for pat in &m.regex_command {
        let re = Regex::new(pat)?;
        if re.is_match(cmd) {
            return Ok(true);
        }
    }
    for pat in &m.regex_path {
        let re = Regex::new(pat)?;
        if re.is_match(path) {
            return Ok(true);
        }
    }

    Ok(false)
}

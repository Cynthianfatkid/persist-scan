use std::path::PathBuf;

/// Expand "~" on unix-like systems; on Windows returns as-is.
pub fn expand_tilde(s: &str) -> PathBuf {
    if !s.starts_with("~/") {
        return PathBuf::from(s);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(s.trim_start_matches("~/"))
}

/// Very conservative "best effort" parsing:
/// - If command starts with quoted string, take that
/// - Else take first token
pub fn extract_executable_path_guess(command: &str) -> Option<String> {
    let c = command.trim();
    if c.is_empty() {
        return None;
    }

    if c.starts_with('"') {
        let rest = &c[1..];
        if let Some(end) = rest.find('"') {
            let p = &rest[..end];
            return Some(p.to_string());
        }
    }

    let first = c.split_whitespace().next()?;
    Some(first.to_string())
}

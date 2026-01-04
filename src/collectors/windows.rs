use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::engine::Artifact;
use crate::util::paths::extract_executable_path_guess;

use winreg::enums::*;
use winreg::RegKey;

pub fn collect_windows() -> Result<Vec<Artifact>> {
    let mut out = vec![];
    out.extend(collect_run_keys()?);
    out.extend(collect_startup_folders()?);
    Ok(out)
}

fn collect_run_keys() -> Result<Vec<Artifact>> {
    let mut out = vec![];

    out.extend(read_run_key(
        HKEY_CURRENT_USER,
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        "HKCU",
    )?);

    out.extend(read_run_key(
        HKEY_LOCAL_MACHINE,
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        "HKLM",
    )?);

    Ok(out)
}

fn read_run_key(root: winreg::HKEY, subkey: &str, scope: &str) -> Result<Vec<Artifact>> {
    let mut out = vec![];
    let root = RegKey::predef(root);

    let key = match root.open_subkey(subkey) {
        Ok(k) => k,
        Err(_) => return Ok(out),
    };

    for item in key.enum_values() {
        let (name, value) = item?;
        let cmd = value.to_string();
        let path_guess = extract_executable_path_guess(&cmd);

        out.push(Artifact {
            kind: "windows_run_key".to_string(),
            source: format!("{}\\{}", scope, subkey),
            name,
            command: Some(cmd),
            path: path_guess,
            raw: None,
        });
    }

    Ok(out)
}

fn collect_startup_folders() -> Result<Vec<Artifact>> {
    let mut out = vec![];

    if let Some(p) = user_startup_folder()? {
        out.extend(list_startup_folder("UserStartup", p)?);
    }

    if let Some(p) = common_startup_folder()? {
        out.extend(list_startup_folder("CommonStartup", p)?);
    }

    Ok(out)
}

fn list_startup_folder(label: &str, folder: PathBuf) -> Result<Vec<Artifact>> {
    let mut out = vec![];
    if !folder.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&folder)
        .with_context(|| format!("Failed reading dir {}", folder.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

        out.push(Artifact {
            kind: "windows_startup_folder".to_string(),
            source: format!("{}:{}", label, folder.display()),
            name,
            command: None,
            path: Some(path.display().to_string()),
            raw: None,
        });
    }
    Ok(out)
}

fn user_startup_folder() -> Result<Option<PathBuf>> {
    let appdata = std::env::var_os("APPDATA").map(PathBuf::from);
    Ok(appdata.map(|p| p.join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")))
}

fn common_startup_folder() -> Result<Option<PathBuf>> {
    let pd = std::env::var_os("ProgramData").map(PathBuf::from);
    Ok(pd.map(|p| p.join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")))
}

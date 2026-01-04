use anyhow::Result;

use crate::engine::{Artifact, TargetOs};

#[cfg(unix)]
mod linux;
#[cfg(windows)]
mod windows;

pub fn collect(os: TargetOs) -> Result<Vec<Artifact>> {
    match os {
        TargetOs::Linux => {
            #[cfg(unix)]
            { linux::collect_linux() }
            #[cfg(not(unix))]
            { anyhow::bail!("Linux collectors not available on this host") }
        }
        TargetOs::Windows => {
            #[cfg(windows)]
            { windows::collect_windows() }
            #[cfg(not(windows))]
            { anyhow::bail!("Windows collectors not available on this host") }
        }
    }
}

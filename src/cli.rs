use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "persist-scan", version, about = "Defensive persistence + suspicious indicator scanner")]
pub struct Args {
    /// Path to rules directory (contains linux/ and windows/)
    #[arg(long, default_value = "./rules")]
    pub rules_dir: String,

    /// Output JSON (for scan command)
    #[arg(long)]
    pub json: bool,

    /// OS selection: auto|linux|windows
    #[arg(long, default_value = "auto")]
    pub os: String,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scan and print findings
    Scan,

    /// Save a baseline snapshot of collected artifacts to a JSON file
    Baseline {
        #[arg(long, default_value = "baseline.json")]
        out: String,
    },

    /// Diff current artifacts vs baseline and print only new/removed artifacts + findings
    Diff {
        #[arg(long, default_value = "baseline.json")]
        baseline: String,
        #[arg(long)]
        show_removed: bool,
        #[arg(long)]
        json: bool,
    },
}

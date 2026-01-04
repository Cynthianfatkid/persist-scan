# persist-scan

persist-scan is a cross-platform, read-only persistence detection tool written in Rust.
It inventories common startup and persistence mechanisms on Linux and Windows, applies a YAML-based rules engine, and highlights system changes using a baseline/diff workflow.

The project is designed for defensive security, blue-team learning, and detection engineering practice, and was validated in isolated Linux and Windows virtual machines using only benign, user-level test artifacts.

### Features
- Cross-platform persistence inventory (Linux & Windows)
- Read-only collectors (no system modification)
- YAML-based, extensible rules engine
- Baseline & diff analysis to detect changes over time
- Human-readable output and JSON reporting
- Designed for safe testing in isolated environments

## What it does
- Collects startup/persistence artifacts (read-only)
  - Linux: `~/.config/autostart/*.desktop`, `crontab -l`
  - Windows: HKCU/HKLM Run keys, Startup folders
- Applies YAML rules to artifacts
- Produces human output or JSON
- Supports baseline + diff to highlight changes over time

## What it does NOT do
- No keystroke capture
- No hooking
- No persistence installation
- No removal / remediation actions

## Commands
- `persist-scan scan`
- `persist-scan baseline --out baseline.json`
- `persist-scan diff --baseline baseline.json`

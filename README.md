# persist-scan

Defensive persistence + suspicious indicator scanner (CLI) with a YAML rules engine.

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

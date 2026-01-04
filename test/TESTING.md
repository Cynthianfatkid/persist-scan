# Testing persist-scan in VMs (safe)

This project is read-only. It inventories startup/persistence artifacts and flags suspicious patterns.
It does not capture keystrokes, hook APIs, or modify system state.

## Build
- Linux:
  - `cargo build --release`
  - run `./target/release/persist-scan scan`
- Windows (PowerShell):
  - `cargo build --release`
  - run `.\target\release\persist-scan.exe scan`

## Baseline / Diff flow
1. Take a clean VM snapshot.
2. Run baseline:
   - `persist-scan baseline --out baseline.json`
3. Make a benign startup change (autostart / cron / Run key / Startup folder).
4. Run diff:
   - `persist-scan diff --baseline baseline.json`

`diff` focuses on what changed compared to baseline (new artifacts, optionally removed ones).

## Linux ideas (manual, benign)
- Create a `.desktop` file in `~/.config/autostart/` with a harmless Exec line that writes to `~/lab_artifacts/*.log`.
- Add a harmless user crontab entry that appends timestamps to a file in `~/lab_artifacts/`.

## Windows ideas (manual, benign)
- Add a user Run key entry (HKCU Run) that launches Notepad or writes a timestamp to `C:\lab_artifacts\*.log`.
- Place a shortcut or script in the Startup folder.

## Cleanup
Remove the items you created, or revert the VM snapshot.

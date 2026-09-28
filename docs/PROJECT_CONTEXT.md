# Project context

## Purpose and current state

RngKit is a Windows-first desktop application for collecting fixed-size samples
from one explicitly selected BitBabbler, TrueRNG, RDSEED, or PseudoRNG source;
monitoring descriptive cumulative statistics; recording native sessions;
generating XLSX reports; and combining compatible current and RngKitPSG v3 CSVs.

The app is a locked Tauri 2 + client-only Svelte 5 + TypeScript + Vite +
Tailwind CSS 4 application. Rust owns coordinator state, discovery tokens,
workers, file jobs, reports, Combine, close policy, preferences, and redacted
diagnostics. Startup prepares `Documents/RngKit` when needed, defaults new
users to 2048 bits, and performs one asynchronous discovery after hydration
without opening or selecting a source. The live chart retains every committed
point and native sessions contain BIN, CSV, and manifest artifacts.

Published app HEAD `a505141` pins reachable `rngkit-core`
`23a67aa4c87d8fa3bbcf049f25786d54966e39d2` (mixed-source concatenation).

## Main product flows

1. **Collect:** discover candidates, require explicit selection, collect until
   cooperative stop, record a native bundle, and plot every committed Z point.
2. **Reports:** inspect native or derived bundles, current standalone CSV/BIN,
   legacy v3 CSV/BIN, or flat canonical legacy concatenation CSVs and write
   same-stem XLSX with explicit Replace.
3. **Combine:** accumulate current, legacy, or mixed-format CSVs across folders
   when bits and interval match exactly, including different sources or folds,
   and create a no-overwrite schema-2 or mixed schema-3 derived bundle without
   changing inputs.
4. **Help:** Quick start, source choice, safe collection, reports, Combine,
   chart interpretation, common problems, and file/version details.

## Architecture and stable constraints

- Tauri owns lifecycle, coordinator, IPC, dialogs, preferences, and artifact
  opening; `rngkit-core` owns adapters, recording, statistics, readers,
  concatenation, and XLSX contents.
- The frontend uses only `core:default` and `dialog:default`; production CSP is
  restricted. Open actions use backend-known paths, never frontend paths.
- One source per session; no silent selection, fallback, live XOR, reconnect,
  or resume. Physical tests are ignored, opt-in, and serial.
- Entropy, seeds, selectors, serials, device paths, and arbitrary diagnostic
  chains never cross IPC or persist. Statistical Z and `+/-1.96` are descriptive
  visual guides, never inference or pass/fail evidence.
- Exact locked floors are Node `^20.19.0 || >=22.12.0`, npm `>=10`, Rust
  edition 2024/MSRV 1.85. Commit, push, publishing a GitHub Release, signing,
  and deployment remain separate approvals. Tag-triggered installer drafts are
  authorized; they do not run on ordinary branch pushes.

## Durable file and workflow contracts

- A native session is same-stem BIN/CSV plus `manifest.json`; CSV is the commit
  marker. A derived bundle is a distinct CSV plus manifest, not a session.
- Reports use one chooser. A present manifest is authoritative; without one,
  standalone current/legacy CSV/BIN metadata is validated from filename and
  contents. Canonical `_concat_` CSVs are a distinct manifest-free legacy
  concatenation kind. Inputs are read-only and existing XLSX requires Replace;
  recorded timestamp versus sample-index chart context is retained from
  inspection and revalidated at generation.
- Combine is CSV-only, accepts current/legacy/mixed-format CSVs when bits and
  interval match exactly, keeps ordered backend paths behind opaque IDs,
  supports Add/Remove/Clear, rejects overlap/incompatibility/BIN, preserves
  schema-1 reading, writes homogeneous schema 2 or mixed schema 3
  `csv_concatenation` with no absolute input paths, and labels mixed output
  `Mixed sources`.
- Help preserves the approved boundary: `Z shows balance over time; it does not
  certify randomness.` Device setup lives under Choosing a source; the kit is
  `src-tauri/resources/device-setup/` mapped to bundled `device-setup`. The app
  never launches Zadig, INF install, or the Linux helper. TrueRNG INF/CAT stay
  excluded pending documented redistribution permission.

## Evidence and open validation

- **Remote CI:** Windows/Ubuntu repair `34002608469`, numeric `34072410813`,
  layout `34077305701`, and `34147766895` (`ad5b35f`) passed (verified 2026-09-09).
- **Historical (2026-08-25):** full frontend, Edge, locked Rust/MSRV and
  no-bundle checks; native PseudoRNG Collect/Stop and manifest-backed XLSX.
  Browser coverage has no Tauri IPC or hardware.
- **User-reported:** 2026-09-07 Help/Reports/Combine and Device setup
  folder-open; 2026-09-10 TrueRNG freshness (`56c0c84` / trng3-rs `08f1889`);
  2026-09-11 mixed Combine via `tauri dev` after pin `23a67aa`, BitBabbler
  acquisition, folds and disconnect, and the installed unsigned NSIS app
  launching and working as expected. The user also reported that the app
  works on Linux; distro, install method, version, and hardware coverage
  were not specified. No separate logs or fold matrix.
- **Remaining:** other artifact open/folder actions, additional hardware,
  native 100k/1M chart interaction, scaling/screen-reader sampling, NSIS
  uninstall/session-data preservation, signing, and publishing a GitHub
  Release draft. Hardware remains opt-in. Linux `.deb` packaging is configured;
  a published Linux release and hardware acceptance are not.

## Windows installer configuration (2026-09-21)

- NSIS remains `.exe`; WebView2 is no longer embedded. `downloadBootstrapper`
  downloads and installs it when missing, requiring internet in that case.
  This supersedes the original offline WebView2 packaging requirement.
- Local locked NSIS build passed: `RngKit_0.1.0_x64-setup.exe` is 8223459 bytes
  (96.3% smaller than the 223732125-byte offline build). SHA-256:
  `5bd2e176c4b1677485e013eee39602ea41833addd4a0f123c931c5a20c0ccda0`.
  7-Zip integrity check passed; listing shows no embedded WebView2 and retains
  ten device-setup files. Generated NSIS uses `downloadBootstrapper`.
  The 12 security tests and changed-file formatting checks passed. The new
  installer has not been installed; present/missing WebView2 setup is untested.
  The artifact evidence below describes the previous offline configuration.

## Unsigned NSIS (2026-09-11)

- Locked NSIS at `a505141` (pin `23a67aa`) passed. The unsigned installer
  `src-tauri/target/release/bundle/nsis/RngKit_0.1.0_x64-setup.exe` is
  223732125 bytes; SHA-256
  `dbc0ca20db58e5bc870a66bcbab92cc424c710459e4d8551bc7f59cab8faab50`.
- 7-Zip inspection verified ten kit files matching source hashes, no TrueRNG
  INF/CAT, and embedded offline WebView2. Generated NSIS installs per user and
  does not launch device-setup tools.
- Non-elevated `/S /UPDATE` exited 0. Installed `rngkit.exe` matches the
  extracted payload SHA-256
  `ffa3a4d8e17344064ca6b7025c663d61e00303725f79e88f01040f0382911f2d`.
  Ten installed kit files match source hashes. Network remained connected.
  The user reported that installation and the installed app work as expected.
  No separate log. Help folder-open was not repeated on this rebuild.
- Update still preserves the previous installation's TrueRNG INF/CAT as extra
  files. Cleanup policy, disconnected installation/folder-open, and
  clean-machine WebView2 remain pending.

## GitHub Release drafts

- `bundle.targets` are `nsis` and `deb`. CI is unchanged (`--no-bundle` on
  `main`/PRs). `.github/workflows/release.yml` packages Windows NSIS and
  Ubuntu 22.04 `.deb` only for tags `v*` or manual `workflow_dispatch`.
  Verify requires matching `v<version>` on tags (internal agreement only on
  branch dispatch) plus successful `ci.yml` Windows and Ubuntu jobs on the
  exact packaged SHA. Tag runs attach installers and `SHA256SUMS.txt` to a
  **draft** Release; prerelease drafts follow the semver prerelease label.
  `workflow_dispatch` from a branch leaves artifacts on the Actions run.
- Rechecked 2026-09-21 before the WebView2 change: origin/main and annotated
  tag `v0.1.0` resolve to `6790926`. Existing unpublished draft `387605724`
  contains Windows NSIS, Linux `.deb`, and `SHA256SUMS.txt`. The user authorized
  commit/push and updating this draft with the smaller Windows installer;
  publishing remains separate.

## README status history (moved)

Moved verbatim from `README.md` on 2026-09-27 when the README was rewritten
for end users. Only the headings were demoted one level; the text is unchanged.

### Original introduction

RngKit is a Windows-first desktop application for collecting entropy samples
from explicitly selected hardware or pseudo-random sources, recording native
sessions, monitoring descriptive cumulative statistics, creating XLSX reports,
and safely combining compatible current and RngKitPSG v3 CSV files.

### Status

The four-destination shell is connected to a Rust coordinator through
discovery, selection, session-draft, preference, and collection commands.
Startup restores safe settings, prepares `Documents/RngKit` when no valid saved
output root exists, and displays 2048-bit new-user defaults. After frontend
hydration, one asynchronous discovery runs without opening or selecting a
source; manual Refresh remains available. Default tests inject fake discovery
and fake sources. Safe settings survive restart. Start opens the
selected source, collects until cooperative Stop, and records a native
BIN/CSV/manifest bundle. Open session folder uses a backend-known path.
Closing while collecting confirms Keep collecting or Stop and exit. Debug
builds include a scenario switch that calls `apply_dev_scenario`; production
omits that command and the switch. The live chart retains every committed
descriptive cumulative Z point. Copied diagnostics are bounded and redacted.
Reports inspect a native session directory, a current or legacy v3 BIN/CSV
file, or a derived concatenation bundle and generate a same-stem XLSX with an
explicit Cancel/Replace round trip. Combine accumulates compatible current,
legacy, or mixed CSV files across folders and creates a provenance-bearing
derived bundle without modifying inputs.
Production capabilities stay `core:default` and `dialog:default` with a
restricted CSP. Open commands use backend-known paths only.

Ignored BitBabbler, TrueRNG, RDSEED, and unified discovery smokes live in
`src-tauri/tests/hardware.rs`; default tests do not run them.

The latest unsigned NSIS build is the local 2026-09-21 configuration; uninstall and
other unverified evidence are listed in `docs/PROJECT_CONTEXT.md`. The reusable library is
[rngkit-core](https://github.com/Thiagojm/rngkit-core) at
`23a67aa4c87d8fa3bbcf049f25786d54966e39d2`.

The approved 2026-08-25 artifact-feedback/report-charts plan and the subsequent
terminal-outcome/local-clock corrections are implemented against the exact
library revision above. App Phase 2 is `b946c4d`, backend Phase 3 is `44e0d65`,
and UI Phase 4 is `b137419`. Native integrated workflow validation remains the
active user gate.

### Sources of truth

- Product contract: `docs/specs/2026-08-22-rngkit-tauri-design.md`
- Execution plan: `docs/plans/2026-08-22-rngkit-tauri-plan.md`
- Approved improvements: `docs/specs/2026-08-24-rngkit-workflow-improvements-design.md`
- Approved phased improvements plan: `docs/plans/2026-08-24-rngkit-workflow-improvements-plan.md`
- Approved artifact-feedback design: `docs/specs/2026-08-25-rngkit-artifact-feedback-and-report-charts-design.md`
- Approved artifact-feedback plan: `docs/plans/2026-08-25-rngkit-artifact-feedback-and-report-charts-plan.md`
- Current state: `docs/PROJECT_CONTEXT.md`
- Durable decisions: `docs/DECISIONS.md`
- Roadmap: `TODO.md`

### Stack

Exact versions are locked in `package-lock.json` and `src-tauri/Cargo.lock`.

- Tauri 2.11.5, `@tauri-apps/cli` 2.11.4, `@tauri-apps/api` 2.11.1
- Svelte 5.56.10, Vite 8.2.2, TypeScript 6.0.3
- Tailwind CSS 4.3.3 via `@tailwindcss/vite`
- uPlot 1.6.32
- Playwright 1.62.1 for browser-level scaffold and later mocked-IPC tests
- Rust edition 2024, MSRV 1.85
- Node.js `^20.19.0 || >=22.12.0`, npm `>=10`

### CI evidence (from Development)

`.github/workflows/ci.yml` runs locked frontend and Rust checks on Windows and
Ubuntu, then `npm run tauri -- build --no-bundle -- --locked`. It does not run ignored
physical tests or build an installer. Observed remote success for `061f66a`:
https://github.com/Thiagojm/rngkit-tauri/actions/runs/32755861549

### Local installer builds (from Packaging)

Local 2026-09-21 build (unsigned, not published):
`src-tauri/target/release/bundle/nsis/RngKit_0.1.0_x64-setup.exe`
8223459 bytes (8.2 MB), 96.3% smaller than the previous offline build. SHA-256
`5bd2e176c4b1677485e013eee39602ea41833addd4a0f123c931c5a20c0ccda0`.
7-Zip integrity and payload inspection passed; WebView2 is not embedded.
This installer has not been installed or tested on a machine missing WebView2.

Historical 2026-09-11 evidence (previous offline configuration):
`src-tauri/target/release/bundle/nsis/RngKit_0.1.0_x64-setup.exe`
223732125 bytes, SHA-256
`dbc0ca20db58e5bc870a66bcbab92cc424c710459e4d8551bc7f59cab8faab50`.
The file is not tracked. Non-elevated `/S /UPDATE` installed that payload on
this Windows host with network connected. The user reported that installation
and the installed app work as expected. Uninstall, session-data preservation,
disconnected install, and clean-machine WebView2 remain unverified. Windows may
warn because the package is unsigned.

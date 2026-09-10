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

Published workflow, artifact-feedback, report-chart, outcome and path corrections
use reachable `rngkit-core` revision `56c0c84d1f51bad1b56e072985459dff264570a9`.

## Main product flows

1. **Collect:** discover candidates, require explicit selection, collect until
   cooperative stop, record a native bundle, and plot every committed Z point.
2. **Reports:** inspect native or derived bundles, current standalone CSV/BIN,
   legacy v3 CSV/BIN, or flat canonical legacy concatenation CSVs and write
   same-stem XLSX with explicit Replace.
3. **Combine:** accumulate compatible current, legacy, or mixed CSVs across
   folders and create a no-overwrite schema-2 derived bundle without changing
   inputs.
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
  edition 2024/MSRV 1.85. Commit, push, release, signing, publication, and
  deployment remain separate approvals.

## Durable file and workflow contracts

- A native session is same-stem BIN/CSV plus `manifest.json`; CSV is the commit
  marker. A derived bundle is a distinct CSV plus manifest, not a session.
- Reports use one chooser. A present manifest is authoritative; without one,
  standalone current/legacy CSV/BIN metadata is validated from filename and
  contents. Canonical `_concat_` CSVs are a distinct manifest-free legacy
  concatenation kind. Inputs are read-only and existing XLSX requires Replace;
  recorded timestamp versus sample-index chart context is retained from
  inspection and revalidated at generation.
- Combine is CSV-only, accepts compatible current/legacy/mixed inputs, keeps
  ordered backend paths behind opaque IDs, supports Add/Remove/Clear, rejects
  overlap/incompatibility/BIN, preserves schema-1 reading, and writes schema 2
  `csv_concatenation` output with no absolute input paths.
- Help preserves the approved boundary: `Z shows balance over time; it does not
  certify randomness.` It documents the default folder, discovery behavior,
  Fit all, standalone inputs, timestamp provenance, mixed Combine, and recovery
  actions in direct task order.

## Evidence and open validation

- **Remote CI:** Windows/Ubuntu repair run `34002608469` and numeric-validation
  run `34072410813` passed. Collect layout run `34077305701` and current-HEAD
  run `34147766895` (`ad5b35f`) passed, verified on 2026-09-09.
- **Historical validation (2026-08-25, Windows):** full frontend, Edge, locked
  Rust/MSRV and no-bundle checks passed; four physical tests were ignored.
  Native PseudoRNG Collect/Stop and manifest-backed XLSX generation/replacement
  passed, including artifact paths and local offset `-03:00`; Excel was not
  visually inspected in that smoke. Browser coverage has no Tauri IPC/hardware.
- **User-reported manual acceptance (2026-09-07):** refreshed Help and Reports/Combine
  work correctly, including standalone current/legacy CSV/BIN, cross-folder
  combinations and XLSX titles, timestamps and charts inspected in Excel.
  This is user-reported evidence, without a separate execution log.
- **Remaining acceptance:** other artifact open/folder actions, hardware/unplug,
  native 100k/1M chart rendering/interaction, scaling/screen-reader sampling,
  NSIS uninstall/session-data preservation, and signing/publication. Hardware
  remains opt-in; the later authorized NSIS evidence is below.

## Current Collect behavior and acceptance (2026-09-06)

- Numeric drafts validate on Start: positive whole bits divisible by 8 and positive
  whole seconds, within u32. Invalid input opens a corrective dialog; Start awaits
  backend acceptance. Terminal actions and source/session details live in Session.
- Compact Monitoring fits 1280x800 in browser Ready/Collecting/Completed scenarios
  without reducing the chart. Smaller windows and expanded content still scroll.
- Frontend checks, 117 unit tests, build and five Edge E2E tests passed. Browser
  layout measurements use simulated state. The user reported successful native
  validation of numeric rules/actions and approved the compact layout.
- Help refresh implemented: responsive topic navigation, Quick start card, current
  numeric/Session guidance, and native troubleshooting/reference disclosures.
  Two Help unit tests, six Edge E2E tests, types/build and visual browser checks
  passed, including keyboard focus, both themes and enlarged text. The user
  reported successful native Help acceptance on 2026-09-07.

## Device setup (2026-09-07)

- Choosing a source includes Device setup with four native disclosures
  (Windows/Ubuntu-Debian × BitBabbler/TrueRNG3). Collect jumps to that heading
  through transient in-memory intent. The kit is `src-tauri/resources/device-setup/`,
  mapped to bundled `device-setup`. Help opens it with no-argument
  `open_device_setup_folder`; the app never launches Zadig, INF install, or the
  Linux helper. Frontend 27 files/127 tests, 7 Edge E2E, locked Rust/MSRV, and
  clippy passed. The user reported successful native folder-open on 2026-09-07.
  Linux hardware acceptance remains pending; folder-open does not establish
  driver installation.
- Review corrections: Linux instructions invoke Bash explicitly; every apply
  reloads udev so a failed reload can be retried with identical rules. TrueRNG
  INF/CAT are removed pending documented redistribution permission; Help points
  to the manufacturer package for advance download.
- Review-fix validation: seven Help unit tests, seven Edge E2E tests, frontend
  check/lint/format and Bash syntax/isolated helper tests passed. The helper
  regression covers repeated reload failure and successful recovery. Git Bash
  skipped symlink/mode cases; this is not native Linux or hardware evidence.
- Both Bash scripts use `CDPATH=''` to fix SC1007. Local syntax/helper checks
  and remote run `34147766895` at `ad5b35f` passed (verified 2026-09-09).

## Corrected NSIS validation (2026-09-09)

- Locked NSIS build at `ad5b35f` passed. The unsigned installer is 270227918 bytes;
  SHA-256 `f4ba396e1dcb32ade4d6d327c7cd65e9156fb7a649b318638e0ab031e1c409ad`.
- 7-Zip inspection/extraction verified ten kit files matching source hashes,
  no TrueRNG INF/CAT, and embedded offline WebView2. Generated NSIS installs
  per user and does not launch device setup tools.
- Non-elevated `/S /UPDATE` installation exited 0. Installed executable matches
  the extracted NSIS payload; ten installed kit files match source hashes.
  Native Collect → Device setup → Open device setup folder opened Explorer at
  `%LOCALAPPDATA%\RngKit\device-setup`. Network remained connected.
- Update preserves the previous installation's TrueRNG INF/CAT as extra files;
  exclusion from the new payload does not remove old copies. Cleanup policy,
  disconnected installation/folder-open, and clean-machine WebView2 remain pending.

## TrueRNG freshness integration (2026-09-09–10)
- Pins rngkit-core `56c0c84`, bringing trng3-rs `08f1889` input purge before
  each acquisition. No frontend or adapter API changes. The user reported
  successful native acceptance of the rebuilt executable on 2026-09-10 after
  the unplug regression. No separate log, exact latency or saved count was supplied;
  host purge does not guarantee immediate detection.
- Integration validation: 134 Rust tests and 127 frontend tests passed; four
  physical tests remained ignored. Locked clippy, Rust 1.85 check, format,
  frontend types/lint and no-bundle release build passed. The rebuilt executable
  is `src-tauri/target/release/rngkit.exe`; the installed app/NSIS is unchanged.

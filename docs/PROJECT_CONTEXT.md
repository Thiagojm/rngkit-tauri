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

`main` includes the published workflow improvements through Phase 6. Help is
task-oriented and the deterministic and production-asset browser suites pass;
native integrated workflow validation and the Phase 6 user-approval gate remain
open. The reusable library is pinned to reachable
`rngkit-core` revision `2cdf311dd206cb5e7320ee520ef1e7a5139cc146`.

## Main product flows

1. **Collect:** discover candidates, require explicit selection, collect until
   cooperative stop, record a native bundle, and plot every committed Z point.
2. **Reports:** inspect native or derived bundles, current standalone CSV/BIN,
   or legacy v3 CSV/BIN and write same-stem XLSX with explicit Replace.
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
  contents. Inputs are read-only and existing XLSX requires Replace.
- Combine is CSV-only, accepts compatible current/legacy/mixed inputs, keeps
  ordered backend paths behind opaque IDs, supports Add/Remove/Clear, rejects
  overlap/incompatibility/BIN, preserves schema-1 reading, and writes schema 2
  `csv_concatenation` output with no absolute input paths.
- Help preserves the approved boundary: `Z shows balance over time; it does not
  certify randomness.` It documents the default folder, discovery behavior,
  Fit all, standalone inputs, timestamp provenance, mixed Combine, and recovery
  actions in direct task order.

## Evidence and open validation

- **Deterministic Phase 6 validation (Windows host):** `npm ci`, Prettier,
  Svelte/TypeScript check, ESLint, Vitest 26 files/100 tests, Playwright 5/5,
  Vite build, cargo fmt/check/test/clippy/doc with locked dependencies, Rust
  1.85 check/test, locked no-bundle Tauri release build, and `git diff --check`
  passed. The four physical tests remained ignored.
- **Browser-integrated:** production-asset Edge tests passed for destination
  navigation, Help headings/copy, accessibility, reduced motion/contrast, and
  minimum-window layout. These tests use no real Tauri IPC or hardware.
- **Phase 6 review corrections:** Help now states startup discovery explicitly,
  documents the compact legacy timestamp form, keeps every stable error code at
  the end, and replaces architecture terminology in the primary workflow with
  direct user actions and outcomes.
- **Native user gate (not passed here):** clean-start defaults/discovery;
  PseudoRNG collection and chart Fit all; standalone legacy/current Reports;
  cross-folder mixed Combine and derived report; Help navigation, theme,
  keyboard use, and minimum window.
- **Still unverified:** native hardware/unplug behavior, native 100k/1M chart
  rendering, scaling/screen-reader sampling, NSIS uninstall/session-data
  preservation, signing/publication, and remote CI. Physical hardware and NSIS
  are outside this Phase 6 authorization.

## Sources of truth

- Approved contract: `docs/specs/2026-08-24-rngkit-workflow-improvements-design.md`
- Approved execution: `docs/plans/2026-08-24-rngkit-workflow-improvements-plan.md`
- Original design/plan: `docs/specs/2026-08-22-rngkit-tauri-design.md` and
  `docs/plans/2026-08-22-rngkit-tauri-plan.md`
- Durable decisions: `docs/DECISIONS.md`; backlog: `TODO.md`

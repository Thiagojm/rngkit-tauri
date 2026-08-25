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

`main` includes the published workflow improvements through Phase 6. The
artifact-feedback/report-chart work and follow-up outcome, local-clock,
selected-basename, and Windows display-path corrections are integrated against
reachable `rngkit-core` revision
`3dc969d983ffa7c981536c46d19afa223f0c490b`.

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

- **Complete deterministic validation (2026-08-25, Windows):** npm install,
  format/check/lint, 27 Vitest files/106 tests, Playwright 5/5, Vite build,
  locked Rust fmt/check/test/clippy/doc, Rust 1.85 check/test, locked no-bundle
  release build, and `git diff --check` passed. Four physical tests remained
  ignored. Production-asset Edge coverage has no real Tauri IPC or hardware.
- **Native smoke (2026-08-25):** startup discovery and 2048-bit default were
  visible; PseudoRNG collection plotted committed samples; Stop immediately
  displayed all saved artifact paths without another UI action. A collected
  manifest previewed and generated/replaced XLSX, and its outcome displayed a
  normal `D:\...` path without the internal `\\?\` prefix. The manifest stored
  local offset `-03:00`. The workbook was not visually inspected in Excel.
- **Remaining native acceptance:** flat/standalone legacy/current and BIN-only
  report variants, selected-BIN chart title, cross-folder mixed Combine and
  derived report, artifact open/folder actions, Help/theme/keyboard/minimum
  window, hardware/unplug behavior, and native 100k/1M chart interaction.
- **Still unverified:** native hardware/unplug behavior, native 100k/1M chart
  rendering, scaling/screen-reader sampling, NSIS uninstall/session-data
  preservation, signing/publication, and remote CI. Physical hardware and NSIS
  are outside this Phase 6 authorization.

## Sources of truth

- Approved historical artifacts remain under `docs/specs/` and `docs/plans/`;
  current contracts and remaining gates are summarized here and in
  `docs/DECISIONS.md` and `TODO.md`.

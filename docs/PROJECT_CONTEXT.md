# Project context

## Purpose and current state

RngKit is a Windows-first desktop application that collects fixed-size entropy samples from one explicitly selected BitBabbler, TrueRNG, RDSEED, or PseudoRNG; monitors descriptive statistics; records native sessions; generates XLSX; and safely combines compatible RngKitPSG v3 CSV files.

This repository has a locked Tauri 2 + client-only Svelte 5 + TypeScript + Vite + Tailwind CSS 4 app with a four-destination shell,
Rust-authoritative coordinator, camel-case DTOs, discovery/selection, a validated session draft,
and a PseudoRNG collection vertical slice. Preferences persist only output
root, sample bits, interval, fold, theme, and physical window geometry.
Default start is idle and does not enumerate hardware or restore a source.
Refresh discovers currently attached families behind opaque tokens; Start
reconstructs `SourceConfig` in Rust, opens the selected source on one worker
thread, and records a native BIN/CSV/manifest bundle until cooperative Stop.
Frontend events carry sequenced metrics only; session/sequence reconciliation
preserves terminal events across concurrent command responses. Open session
folder accepts a backend-known directory, not a frontend path. Debug builds
expose `apply_dev_scenario`; release builds omit that command. The live uPlot chart
retains every committed `(sample_index, cumulative_z)` point with coalesced
redraws, zoom and theme-change viewport persistence, Reset view, and Return to
live. Close while collecting confirms Keep collecting or Stop and exit; close
while stopping waits for cooperative finalization. Active sessions reloaded in
the frontend poll authoritative state until terminal reconciliation. Copy
diagnostics returns a redacted in-memory history. Ignored `hardware.rs` smokes require explicit tokens;
this Windows host passed BitBabbler White fold-0, TrueRNG, and RDSEED ordinal 1 (3 fake-clock samples each), then listed those families and PseudoRNG without opening.
Reports inspect a chosen native directory, read-only legacy v3 BIN/CSV, or derived concatenation bundle in Rust, expose recorded/estimated timestamp provenance, reject live/corrupt/unsupported inputs, and generate same-stem XLSX through `write_report()`; Cancel/Replace are separate requests, and artifacts open only when ready and idle. Combine previews compatible legacy v3 CSV files, creates a no-overwrite derived CSV/manifest bundle, and never stores absolute input paths.
Capabilities remain `core:default` and `dialog:default`. Production CSP excludes development endpoints; open/reveal IPC takes no frontend path and Windows artifact launching bypasses command interpreters. Adversarial tests cover malformed derived manifests, no-overwrite XLSX, and Windows junctions; file-symlink creation needs a privilege and is unverified. A release Tauri build without bundling passed on this host. Remote CI on
`windows-latest` and `ubuntu-22.04` succeeded for `a9f99e5` at
https://github.com/Thiagojm/rngkit-tauri/actions/runs/32750338632 (no hardware
jobs, no installer). Ubuntu remains compile evidence, not Linux desktop support.
An unsigned per-user English NSIS installer with offline WebView2 exists locally
as `src-tauri/target/release/bundle/nsis/RngKit_0.1.0_x64-setup.exe` (208.4 MiB,
SHA-256 `612BC8F006FA974AE961DDDB4348CE29E8ACBFB7758EF7A7683D6F8B8DDE8DE7`);
install, uninstall, SmartScreen, signing, and publication remain unverified or out of scope.

The reusable library is public at `https://github.com/Thiagojm/rngkit-core`.
The app pins git revision `183f3c7811f5593b3b42c2558ac726552b86687d`, which
contains derived concatenation inspect/create/`open_concatenation`/XLSX and
entropy-free PseudoRNG discovery.

## Main product flows

1. **Collect:** discover candidates, require explicit selection, configure one
   source, collect until cooperative stop, record a native bundle, and display
   every committed cumulative Z point.
2. **Reports:** inspect validated native, legacy v3, or derived inputs and create
   same-stem XLSX reports with explicit replacement confirmation.
3. **Combine:** preview compatible legacy v3 CSV files and create a provenance-
   bearing no-overwrite CSV plus manifest bundle without modifying inputs.
4. **Help:** explain sources, folds, formats, troubleshooting, and descriptive
   statistical limits.

## Architecture boundary

- Tauri owns desktop lifecycle, the Rust-authoritative coordinator, worker
  threads, IPC, dialogs, preferences, artifact opening, and safe errors.
- `rngkit-core` crates own source adapters, collection, recording, statistics,
  normalized readers, strict concatenation, and XLSX contents.
- Svelte consumes typed application DTOs and never becomes authoritative for
  collection state, filesystem safety, or statistical calculations.
- Collection uses one application-owned worker and a per-session ordered
  channel with session and sequence filtering; worker startup, source open,
  engine, and terminal delivery failures reconcile to backend `failed` state.

## Domain terms

- **Native session:** same-stem BIN/CSV files plus `manifest.json`; CSV is the
  commit marker.
- **Derived concatenation:** a distinct same-stem CSV plus manifest bundle with
  hashes and input/output provenance; it is not a collected session.
- **Cumulative Z:** descriptive `(2*C - N) / sqrt(N)` value.
- **Reference +/-1.96:** visual chart guides only, never a significance result.
- **Candidate token:** transient opaque backend identifier for one discovered
  source; it is invalidated by refresh and never persisted.

## Stable constraints

- Windows 10/11 x64 is the v1 desktop target; Ubuntu CI is not Linux desktop support.
- Locked stack: Tauri 2.11.5, Svelte 5.56.10, TypeScript 6.0.3, Vite 8.2.2,
  Tailwind CSS 4.3.3, uPlot 1.6.32. Prereleases and extra UI frameworks are
  excluded. Playwright 1.62.1 provides browser-level tests against production
  assets. Node floor is `^20.19.0 || >=22.12.0`; npm `>=10`.
- CSS-first Tailwind tokens retain light defaults; plain CSS media overrides
  apply the system dark palette without collapsing it into the global theme.
- Rust edition 2024 and MSRV 1.85 match the library workspace.
- `rngkit-*` crates use git revision `183f3c7811f5593b3b42c2558ac726552b86687d`,
  never a local path. Discovery advertises compiled PseudoRNG capability and
  defers OS entropy acquisition to explicit source opening.
- Frontend capabilities are `core:default` and `dialog:default` only.
- One source per session; no live XOR, fallback, reconnect, resume, or silent
  first-device selection.
- Retain every chart point for the active session. The 2026-08-24 Windows
  data-only harness retained 100,001 points after append (0.29 ms replace,
  0.47 ms append, 7.7 MiB heap delta) and 1,000,001 points (1.76 ms replace,
  7.27 ms append, 31.1 MiB delta). Native canvas render and interaction at
  those sizes remain unverified.
- Diagnostics and preferences must exclude entropy, seeds, selectors, serials,
  device paths, and absolute legacy input paths. Frontend error messages are
  canonical safe strings; raw failure detail is redacted before retention.
- v1 packaging is an unsigned per-user English NSIS installer with offline
  WebView2 (`com.rngkit.desktop`, current-user, English only). The embedded
  WebView2 offline installer adds about 127 MB. Signing, release, updater, and
  deployment remain separate work.
- Validation claims must distinguish deterministic, CI, native Windows,
  physical hardware, chart stress, and installer evidence. Physical results
  name device variant, ordinal, fold, and OS; absence is unverified, not passed.

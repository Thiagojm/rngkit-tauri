# Project context

## Purpose and current state

RngKit is a Windows-first desktop application that collects fixed-size entropy
samples from one explicitly selected BitBabbler, TrueRNG, RDSEED, or PseudoRNG;
monitors descriptive statistics; records native sessions; generates XLSX; and
safely combines compatible RngKitPSG v3 CSV files.

This repository is a locked Tauri 2 + client-only Svelte 5 + TypeScript + Vite +
Tailwind CSS 4 app. Rust owns the coordinator
(`idle|discovering|ready|collecting|stopping|completed|failed`), file jobs,
discovery tokens, collection worker, reports, Combine, close policy, and
redacted diagnostics. Startup prepares `Documents/RngKit` when no valid saved
root exists, defaults new users to 2048 bits, and performs one asynchronous
discovery after frontend hydration without selecting a source.
Production IPC matches the approved command list; `apply_dev_scenario` is
debug-only. Capabilities are `core:default` and `dialog:default`. The live
chart retains every committed `(sample_index, cumulative_z)` point. v1
packaging is an unsigned per-user English NSIS installer with offline WebView2.

The product-code and installer baseline remains `061f66a`; current `main` also
contains the completed Phase 2 implementation. The Checkpoint 18 audit is
complete.
Original design acceptance criteria 1–17 and 19–20 are evidenced. Criterion 18
is partial: offline installation, first launch, and basic app functionality
passed by user report; uninstall and session-data preservation remain
unverified. No required original-v1 product work is stubbed.

The follow-on workflow-improvements design and phased plan dated 2026-08-24
are approved and readable under `docs/specs/` and `docs/plans/`. Phase 1 is
complete and published in `rngkit-core`; Phase 2 is complete and validated
automatically in this app. Phase 3 is implemented in the current worktree and
has passed automated validation; its native manual test is the active gate and
Phase 4 is not authorized.

The library is `https://github.com/Thiagojm/rngkit-core` at
`2cdf311dd206cb5e7320ee520ef1e7a5139cc146` (git, never a local path).

## Main product flows

1. **Collect:** discover candidates, require explicit selection, collect until
   cooperative stop, record a native bundle, and plot every committed Z point.
2. **Reports:** inspect native, legacy v3, or derived inputs and write same-stem
   XLSX with explicit Replace.
3. **Combine:** preview compatible legacy v3 CSVs and create a no-overwrite
   derived CSV/manifest bundle without modifying inputs.
4. **Help:** sources, folds, formats, troubleshooting, and descriptive limits.

## Architecture boundary

- Tauri owns lifecycle, coordinator, workers, IPC, dialogs, preferences, and
  artifact opening.
- `rngkit-core` crates own adapters, collection, recording, statistics, readers,
  concatenation, and XLSX contents.
- Svelte consumes camel-case DTOs and is never authoritative for collection,
  filesystem safety, or statistics.

## Domain terms

- **Native session:** same-stem BIN/CSV plus `manifest.json`; CSV is the commit
  marker.
- **Derived concatenation:** distinct same-stem CSV plus manifest; not a
  collected session.
- **Cumulative Z:** descriptive `(2*C - N) / sqrt(N)`.
- **Reference +/-1.96:** visual guides only, never a significance result.
- **Candidate token:** transient opaque backend id; invalidated by refresh.

## Stable constraints

- Windows 10/11 x64 is the v1 desktop target; Ubuntu CI is compile evidence.
- Locked stack: Tauri 2.11.5, Svelte 5.56.10, TypeScript 6.0.3, Vite 8.2.2,
  Tailwind CSS 4.3.3, uPlot 1.6.32, Playwright 1.62.1. Node
  `^20.19.0 || >=22.12.0`; npm `>=10`. Rust edition 2024, MSRV 1.85.
- Frontend capabilities stay `core:default` and `dialog:default`.
- One source per session; no live XOR, fallback, reconnect, resume, or silent
  first-device selection.
- Diagnostics and preferences exclude entropy, seeds, selectors, serials,
  device paths, and absolute legacy input paths.
- Statistical Z and `+/-1.96` stay descriptive, never inferential.
- Signing, publication, updater, release, and deployment need separate
  approval.

## Evidence (2026-08-24, Checkpoint 18)

- **Deterministic (this Windows host):** `format:check`, `check`, `lint`,
  Vitest 91/91, Playwright 5/5, `vite build`, cargo fmt/check/test/clippy/doc
  and `+1.85.0` check/test, all locked, `git diff --check`. Hardware tests
  compiled and stayed ignored. Chart data-only harness: 100,001 points (0.28 ms
  replace, 0.43 ms append, 7.7 MiB heap delta) and 1,000,001 points (1.70 ms
  replace, 7.14 ms append, 31.7 MiB). Tracked tree has no installer, session
  data, secret, or `path=` crate pin.
- **CI:** `windows-latest` and `ubuntu-22.04` for `061f66a` at
  https://github.com/Thiagojm/rngkit-tauri/actions/runs/32755861549 (no
  hardware jobs, no installer). Ubuntu is not Linux desktop support.
- **Physical (Windows, ignored, serial, 3 fake-clock samples):** BitBabbler
  White fold-0, TrueRNG, RDSEED ordinal 1; unified discovery then listed those
  families and PseudoRNG without opening.
- **Installer:** local unsigned
  `src-tauri/target/release/bundle/nsis/RngKit_0.1.0_x64-setup.exe` (208.4 MiB,
  SHA-256 `612BC8F006FA974AE961DDDB4348CE29E8ACBFB7758EF7A7683D6F8B8DDE8DE7`),
  not tracked. The user reported offline installation, first launch, and basic
  app functionality on Windows. Uninstall, session-data preservation, and
  SmartScreen behavior remain unverified.

## Unverified (not passed)

Native Collect on hardware, unplug-during-read, other folds/devices, Linux
physical; native 100k/1M chart canvas render/interaction; native
Reports/Combine dialogs; Windows 100%/150%/200% scaling and screen-reader
sampling; Windows file-symlink inspect (privilege 1314); NSIS uninstall and
session-data preservation; SmartScreen behavior, signing, and publication.

## Evidence (2026-08-24, workflow improvements Phase 2)

- **Deterministic (this Windows host):** locked npm install, format, Svelte and
  TypeScript checks, lint, Vitest 92/92, Playwright 5/5, production Vite build,
  locked cargo fmt/check/test/clippy/doc, and Rust 1.85 check/test passed.
- Default tests compiled four physical smokes and kept them ignored. No native
  window, physical source, installer, or remote CI check was run for Phase 2.
- Focused recovery tests cover clean default creation, preservation of a valid
  custom root and sample size, missing-root fallback, unavailable Documents,
  the combined failure case, and clearing a stale warning after choosing a
  valid folder. Security temp roots use a process-local counter to avoid
  parallel Windows test collisions.

## Evidence (2026-08-24, workflow improvements Phase 3)

- The chart now has one `Fit all` action, adapter-owned following state,
  supersedable animation-frame updates, and pointer zoom/pan pause behavior.
  The instrument-style card is taller and keeps the concise descriptive
  boundary copy; retention remains every committed point.
- Deterministic validation passed: format check, Svelte/TypeScript check, lint,
  Vitest 95/95, Playwright 5/5, production Vite build, chart stress 100k/1M,
  cargo fmt/check/test/clippy/doc, Rust 1.85 check/test, and `git diff --check`.
- Chart stress measurements were 100,000 points: 0.40 ms replace, 0.567 ms
  append, 7.9 MiB heap delta; 1,000,000 points: 2.21 ms replace, 8.002 ms
  append, 30.9 MiB heap delta.
- Browser CLI snapshot/screenshot evidence confirmed one `Fit all` control,
  the boundary copy, no legacy Reset/Return controls, and the larger chart
  card. Native window interaction remains unverified and is the user gate.

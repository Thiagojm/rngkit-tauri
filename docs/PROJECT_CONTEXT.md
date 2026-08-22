# Project context

## Purpose and current state

RngKit is a planned Windows-first desktop application for collecting fixed-size
entropy samples from one explicitly selected BitBabbler, TrueRNG, RDSEED, or
PseudoRNG source, monitoring descriptive cumulative statistics, recording
native sessions, generating XLSX reports, and safely combining compatible
RngKitPSG v3 CSV files.

This repository currently contains an approved design, an approved staged
implementation plan, and repository context only. It has no application
scaffold, executable, package manifests, lockfiles, or verified build commands.

The reusable library is public at `https://github.com/Thiagojm/rngkit-core`.
Commit `954125cc9a664d372c4ed4a39656b790d21ba333` completed and validated
Checkpoint 1 (derived naming, manifest contracts, and legacy CSV inspection).
Checkpoint 2 (bundle creation, reading, and derived XLSX) is still required
before the app can pin the final library revision and begin scaffolding.

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
  channel with session and sequence filtering.

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

- Windows 10/11 x64 is the v1 desktop target; Ubuntu CI is not Linux desktop
  support.
- Planned stack: Tauri 2, Svelte 5, TypeScript, Vite, Tailwind CSS 4, and uPlot;
  select the latest mutually compatible stable versions at scaffolding time and
  lock them exactly.
- Rust edition 2024 and MSRV 1.85 must match the library workspace.
- One source per session; no live XOR, fallback, reconnect, resume, or silent
  first-device selection.
- Retain every chart point for the active session and measure 100,000 and
  1,000,000 points before claiming long-session performance.
- Frontend capabilities stay minimal: no general filesystem, shell, opener, or
  logging access.
- Diagnostics and preferences must exclude entropy, seeds, selectors, serials,
  device paths, and absolute legacy input paths.
- v1 packaging is an unsigned per-user English NSIS installer with offline
  WebView2. Signing, release, updater, and deployment remain separate work.
- Validation claims must distinguish deterministic, CI, native Windows,
  physical hardware, chart stress, and installer evidence.

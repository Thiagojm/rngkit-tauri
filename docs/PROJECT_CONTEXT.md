# Project context

## Purpose and current state

RngKit is a Windows-first desktop application for collecting fixed-size
entropy samples from one explicitly selected BitBabbler, TrueRNG, RDSEED, or
PseudoRNG source, monitoring descriptive cumulative statistics, recording
native sessions, generating XLSX reports, and safely combining compatible
RngKitPSG v3 CSV files.

This repository has a locked Tauri 2 + client-only Svelte 5 + TypeScript +
Vite + Tailwind CSS 4 app with the four-destination shell, a Rust-authoritative
coordinator, camel-case DTOs, `get_app_state`, `refresh_sources`, and
`select_source`. Default start is idle and does not enumerate hardware.
Refresh discovers currently attached families behind opaque tokens; nothing is
selected automatically. Debug builds expose `apply_dev_scenario`; release
builds omit that command. Collection and session files are not connected yet.

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
- Retain every chart point for the active session and measure 100,000 and
  1,000,000 points before claiming long-session performance.
- Diagnostics and preferences must exclude entropy, seeds, selectors, serials,
  device paths, and absolute legacy input paths. Frontend error messages are
  canonical safe strings; raw failure detail is redacted before retention.
- v1 packaging is an unsigned per-user English NSIS installer with offline
  WebView2. Signing, release, updater, and deployment remain separate work.
- Validation claims must distinguish deterministic, CI, native Windows,
  physical hardware, chart stress, and installer evidence.

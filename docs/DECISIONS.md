# Decisions

All decisions are accepted. Material changes return to design review.

## Product and platform

- RngKit v1 is an English Windows 10/11 x64 Tauri 2 desktop app with persistent
  Collect, Reports, Combine, and Help destinations.
- The client-only Svelte 5/Vite/Tailwind CSS 4 frontend uses locked stable
  dependencies, uPlot, no extra UI framework, `data-theme`, reduced-motion
  support, 4.5:1 body/muted contrast, and an 800x600 minimum window.
- Browser tests use production assets through Edge without real Tauri IPC or
  hardware. Debug scenario switching is not a production capability.

## Authority, lifecycle, and IPC

- Rust owns coordinator states (`idle`, `discovering`, `ready`, `collecting`,
  `stopping`, `completed`, `failed`), workers, file-job exclusion, discovery
  tokens, session IDs, event sequences, preferences, and close policy.
- One explicitly selected source serves a session. Refresh invalidates tokens;
  discovery never opens a source; Start reconstructs configuration from a
  transient token; Stop is cooperative and idempotent. No silent selection,
  fallback, XOR, reconnect, resume, or force quit.
- Production IPC is `get_app_state`, `refresh_sources`, `select_source`,
  `set_sample_bits`, `set_interval_seconds`, `set_fold`, `set_theme`,
  `choose_output_folder`, `start_collection`, `stop_collection`,
  `start_another_session`, `open_session_folder`, `copy_diagnostics`,
  `stop_and_exit`, `choose_report_input`, `generate_report`, `replace_report`,
  `open_report`, `open_report_folder`, `choose_csv_inputs`,
  `remove_combine_input`, `clear_combine_inputs`, `create_derived`,
  `generate_derived`, and `open_derived_folder`. `apply_dev_scenario` is
  debug-only.
- Per-session events expose numeric cumulative Z and safe labels only. The
  frontend retains every committed point; zero and `+/-1.96` are visual guides.

## Privacy and filesystem authority

- Capabilities are only `core:default` and `dialog:default`; CSP is restricted.
  Open actions accept no frontend path and use backend-known destinations.
- Safe errors and bounded diagnostics redact paths, ports, selectors, serials,
  seeds, entropy, and arbitrary error chains. Preferences contain only safe
  settings and physical window geometry.
- Native and derived artifacts enforce containment, no-follow, no-overwrite,
  and revalidation after preview. Inputs remain unmodified.

## Reports and Combine

- Reports has one chooser for CSV, BIN, and JSON. A manifest/bundle is resolved
  and classified authoritatively; absent manifests use the published core
  standalone reader for current/legacy CSV/BIN. Same-stem XLSX generation uses
  explicit Cancel/Replace and preserves input bytes. The selected artifact
  basename remains authoritative even when a `.bin` uses a valid CSV sibling
  for recorded timestamps.
- Combine is CSV-only and accepts distinct compatible current, legacy, or mixed
  inputs. Ordered canonical paths stay backend-only behind transient opaque
  input IDs. Add appends, Remove targets one row, Clear resets selection, and
  changed/duplicate/overlap/incompatible/BIN inputs fail safely.
- Schema-1 derived bundles remain readable. New output is schema 2,
  `csv_concatenation`, with per-input format metadata and no absolute paths.
- Cumulative Z is `(2*C - N) / sqrt(N)` for descriptive monitoring only; no
  p-values, confidence claims, or statistical pass/fail interpretation.

## Approved workflow improvements

- New users get backend-prepared `Documents/RngKit` and 2048 bits when valid
  saved preferences do not win. Startup discovery is asynchronous and never
  selects a source. Collect has one `Fit all`; zoom/pan pauses following and
  Fit all resumes it only while collecting.
- Help is task-oriented in this order: Quick start, Choosing a source,
  Collecting and stopping safely, Creating reports, Combining files,
  Understanding the chart, Common problems, File formats and version details.
  It states the exact non-certification boundary and recovery actions.

## Artifact feedback and current evidence

- The app pins reachable `rngkit-core` revision
  `3dc969d983ffa7c981536c46d19afa223f0c490b`.
- Artifact feedback is transient and typed: one
  monotonic-ID notice is pending at a time, stale acknowledgements fail, and
  only confirmed regular files/directories under backend-known roots may be
  exposed or opened. One severity-aware outcome dialog renders selectable complete
  paths and only backend-approved actions. It suppresses repeated IDs across
  acknowledgement, polling, hydration, and navigation, while Replace and
  close-collection dialogs retain precedence. User-visible Windows paths omit
  internal extended-length prefixes; internal canonical paths are unchanged.
- A terminal collection channel event immediately reloads backend state so the
  already-created success/failure outcome is rendered without waiting for a
  later UI command. Session ID, channel generation, and event sequence guards
  prevent stale terminal reloads from replacing a newer collection.
- Recorded-time report charts use the local clock contract published in
  the pinned core revision:
  native manifests provide the offset, manifest-free current CSVs infer it from
  the canonical filename, and legacy CSV clocks are not shifted twice.
- Flat canonical `_concat_` CSVs are a distinct Reports kind without a
  manifest. Inspected reports retain the library's source basename and chart
  axis mode in backend-only state; generation revalidates that context before
  writing XLSX.
- Complete deterministic/browser/MSRV/no-bundle validation and native
  PseudoRNG Collect plus manifest-backed Reports smoke validation passed.
  Other native variants remain explicit acceptance work. Default tests remain
  hardware-free; physical smokes are ignored, opt-in, and serial.

## Delivery boundaries

- Locked versions live in `package-lock.json` and `src-tauri/Cargo.lock`.
  Floors are Node `^20.19.0 || >=22.12.0`, npm `>=10`, Rust edition 2024/MSRV
  1.85; prereleases and local crate paths are forbidden.
- v1 packaging is an unsigned per-user English NSIS installer with offline
  WebView2. Uninstall/session-data preservation, SmartScreen, signing,
  publication, updater, release, and deployment remain unverified or separate
  approvals.

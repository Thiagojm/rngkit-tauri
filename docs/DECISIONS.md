# Decisions

All decisions are accepted. Material changes return to design review.

## Product, platform, and shell (2026-08-22)

- RngKit v1 is an English Windows 10/11 x64 Tauri 2 desktop app with one
  resizable window and persistent Collect, Reports, Combine, and Help
  destinations.
- The client-only Svelte 5/Vite/Tailwind CSS 4 frontend uses locked stable
  dependencies, uPlot, and no extra UI framework.
- Theme is `data-theme` on `html`; Tailwind owns light defaults and plain CSS
  preserves system dark unless light is forced. Body and muted text meet 4.5:1
  contrast; reduced-motion disables extra animation; the window minimum is
  800×600.
- Browser tests use production assets through Edge without Tauri IPC or
  hardware. Mock snapshots are browser-only; scenario switching is debug-only.
- Why/impact: modernize familiar workflows while keeping desktop authority and
  permissions out of the frontend.

## Authority, collection, and IPC (2026-08-22 through 2026-08-23)

- Rust owns coordinator states (`idle`, `discovering`, `ready`, `collecting`,
  `stopping`, `completed`, `failed`), file-job exclusion, session IDs, and
  event sequences. It rejects stale discovery/events, double starts, and
  conflicts.
- One explicitly selected source and one application worker serve each session.
  No silent selection, live XOR, fallback, reconnect, or resume is allowed.
- Start reconstructs `SourceConfig` from a transient token, opens the source on
  the worker, and calls `run_session`. Stop is cooperative and idempotent.
  Worker startup, source open, engine, and terminal channel failures finish the
  matching coordinator session as failed; clean stop finishes it as completed.
- Per-session channels carry ordered metric DTOs only. Frontend
  command-response generations do not invalidate terminal channel events.
- Sample-committed events include numeric `cumulativeZ` plus the display label.
  Svelte retains every accepted `(sample_index, cumulative_z)` point. uPlot
  draws zero and dashed ±1.96 references without extra point arrays.
- Close while collecting is intercepted: Keep collecting cancels the close;
  Stop and exit cooperatively stops and waits for the worker before destroy.
  Close while stopping is idempotent. v1 has no force quit.
- Production IPC is `get_app_state`, `refresh_sources`, `select_source`,
  `set_sample_bits`, `set_interval_seconds`, `set_fold`, `set_theme`,
  `choose_output_folder`, `start_collection`, `stop_collection`,
  `start_another_session`, `open_session_folder`, `copy_diagnostics`,
  `stop_and_exit`, `choose_report_input`, `generate_report`, `replace_report`,
  `open_report`, `open_report_folder`, `choose_csv_inputs`, `create_derived`,
  `generate_derived`, and `open_derived_folder` against backend-known paths
  only. `apply_dev_scenario` is debug-only.
- Why/impact: preserve engine durability, deterministic authority, and an
  entropy-free frontend.

## Discovery, draft, and preferences (2026-08-23)

- `refresh_sources` runs `rngkit_sources::discover()` in blocking Tauri work.
  Candidates live behind random opaque tokens. Refresh invalidates prior tokens
  and selection. Discovery never opens a source.
- Preferences schema 1 stores output root, sample bits, interval, fold, theme,
  and physical window geometry only. Writes use a sibling temp plus atomic
  replace. Invalid files reset wholly with a safe warning.
- Ready requires valid bits, interval, fold, output root, and explicit
  selection. Default tests inject fake discovery/sources.
- Why/impact: explicit multi-device selection and restart-safe drafts without
  leaking selectors.

## Privacy, filesystem, and diagnostics (2026-08-22)

- Frontend capabilities are only `core:default` and `dialog:default`.
  Production CSP allows only required app/Tauri protocols.
- Frontend errors are stable `SafeError` DTOs. Diagnostics are bounded,
  redacted in-memory records. Production persistent logging is disabled.
- Entropy, seeds, selectors, serials, OS paths, absolute legacy input paths,
  and arbitrary error chains never cross IPC, preferences, or diagnostics.
- Native and derived artifacts retain containment and no-follow/no-overwrite
  behavior. Open commands take no frontend path.
- Why: keep sensitive material and filesystem authority on the desktop side.

## Reports and derived data (2026-08-22)

- XLSX reports use normalized readers for native sessions, read-only RngKitPSG
  v3 BIN/CSV, and validated derived bundles. Existing output needs explicit
  Replace.
- Strict concatenation accepts distinct compatible legacy v3 CSV inputs,
  rejects ambiguous overlap, revalidates after preview, and never stores
  absolute input paths.
- Statistical Z and `+/-1.96` are descriptive visual references only.

## Dependencies and delivery (2026-08-22 through 2026-08-24)

- Exact versions live in lockfiles. Node floor is `^20.19.0 || >=22.12.0`, npm
  `>=10`, Rust edition 2024/MSRV 1.85. Prereleases are excluded.
- `rngkit-*` crates use reachable revision
  `183f3c7811f5593b3b42c2558ac726552b86687d`, never local paths.
- Default tests are deterministic and hardware-free. Physical checks are
  ignored, opt-in, and serial in `src-tauri/tests/hardware.rs`.
- v1 delivery is an unsigned per-user English NSIS installer with bundled
  offline WebView2, id `com.rngkit.desktop`, no updater artifacts, and no
  certificate. Local `RngKit_0.1.0_x64-setup.exe` is 208.4 MiB, SHA-256
  `612BC8F006FA974AE961DDDB4348CE29E8ACBFB7758EF7A7683D6F8B8DDE8DE7`. The
  user reported offline installation, first launch, and basic app functionality
  on Windows; uninstall and session-data preservation remain unverified.
  Signing, publication, updater, release, and deployment require separate
  approval.
- CI (`windows-latest` and `ubuntu-22.04`) uses `npm ci`, `cargo --locked`, and
  `tauri build --no-bundle -- --locked`; it never runs ignored hardware tests
  or builds an installer. Observed remote success for `061f66a`:
  https://github.com/Thiagojm/rngkit-tauri/actions/runs/32755861549
- Checkpoint 18 (2026-08-24) traced design acceptance criteria 1–20: 1–17 and
  19–20 evidenced; 18 remains partial pending uninstall/session-data
  preservation. No contract change.

# Decisions

All decisions are accepted. Material changes return to design review.

## Product, platform, and shell (2026-08-22)

- RngKit v1 is an English Windows 10/11 x64 Tauri 2 desktop app with one resizable window and persistent Collect, Reports, Combine, and Help destinations.
- The client-only Svelte 5/Vite/Tailwind CSS 4 frontend uses locked stable dependencies, uPlot, and no extra UI framework.
- The scaffold combines official Tauri 2 Rust/config/icons with the Vite `svelte-ts` SPA; the official Svelte Tauri template is SvelteKit.
- Theme is `data-theme` on `html`; Tailwind owns light defaults and plain CSS preserves system dark unless light is forced.
- The top bar exposes product, operation status, and light/dark/system theme. Collect stacks at the 800px minimum; Start and Stop remain separate and disabled controls explain why.
- Browser tests use production assets through Edge without Tauri IPC or hardware. Mock snapshots are browser-only; scenario switching is debug-only.
- Why/impact: modernize familiar workflows while keeping desktop authority and permissions out of the frontend.

## Authority, collection, and IPC (2026-08-22 through 2026-08-23)

- Rust owns coordinator states (`idle`, `discovering`, `ready`, `collecting`,
  `stopping`, `completed`, `failed`), file-job exclusion, session IDs, and event
  sequences. It rejects stale discovery/events, double starts, and conflicts.
- One explicitly selected source and one application worker serve each session.
  No silent selection, live XOR, fallback, reconnect, or resume is allowed.
- Start reconstructs `SourceConfig` from a transient token, opens the source on
  the worker, and calls `run_session`. Stop is cooperative and idempotent.
  Worker startup, source open, engine, and terminal channel failures finish the
  matching coordinator session as failed; clean stop finishes it as completed.
- Per-session channels carry ordered metric DTOs only. Frontend command-response
  generations do not invalidate terminal channel events; session and sequence
  checks prevent older responses or stale events from replacing newer state.
- Sample-committed events include numeric `cumulativeZ` plus the display label.
  Svelte retains every accepted `(sample_index, cumulative_z)` point in aligned
  arrays and never receives entropy or calculates authoritative statistics.
  uPlot draws zero and dashed ±1.96 references without extra point arrays and
  coalesces redraws to one animation frame. Appends and theme redraws preserve
  user zoom and the mounted plot; Reset fits once and Return to live resumes fit.
- Close while collecting is intercepted: Keep collecting cancels the close;
  Stop and exit cooperatively stops and waits for the worker before destroy.
  The close policy is captured while coordinator state is locked. Close while
  stopping is idempotent and shows finalization; v1 has no force quit. Channel
  loss is terminal and queryable; an active frontend reload polls
  `get_app_state` until terminal reconciliation. Copied diagnostics are redacted
  in-memory records.
- Production IPC is `get_app_state`, `refresh_sources`, `select_source`,
  `set_sample_bits`, `set_interval_seconds`, `set_fold`, `set_theme`,
  `choose_output_folder`, `start_collection`, `stop_collection`,
  `start_another_session`, `open_session_folder`, `copy_diagnostics`,
  `stop_and_exit`, `choose_report_input`, `generate_report`, `replace_report`, `open_report`,
  `open_report_folder`, `choose_csv_inputs`, `create_derived`, `generate_derived`, and `open_derived_folder` against backend-known paths only;
  collection and file jobs block artifact opening. `apply_dev_scenario` is debug-only; browser tests use mock snapshots.
- Why/impact: preserve engine durability, deterministic authority, responsive
  UI, and an entropy-free frontend.

## Discovery, draft, and preferences (2026-08-23)

- `refresh_sources` runs `rngkit_sources::discover()` in blocking Tauri work.
  A backend-only generation registry holds candidates behind random opaque tokens;
  DTOs expose token, source id, safe label, variant, ordinal, and fold requirement only.
  Refresh invalidates prior tokens and selection.
- Multiple devices remain separate. Partial family failures are safe warnings.
  Discovery never opens a source; compiled PseudoRNG is capability only, and OS
  entropy availability is authoritative at explicit `open()`.
- Rejected refresh/selection reconciles through `get_app_state`, restores usable controls, and exposes only structured safe errors.
- Preferences schema 1 contains output root, sample bits, interval, fold, theme,
  and physical window geometry. It never stores selection, tokens, families,
  serials, device paths, entropy, or seeds.
- Writes use a sibling temporary file plus atomic platform replacement and roll
  back both in-memory authorities on failure. Invalid/unsupported files reset
  wholly with a safe warning without touching session files.
- The directory dialog runs in Rust and returns only a label. Restored roots are
  revalidated; missing roots are dropped. Physical geometry is clamped to a
  visible monitor in one mixed-DPI-safe coordinate space.
- Ready requires valid bits, interval, fold, output root, and explicit selection.
- Default tests inject fake discovery/sources and never enumerate or open hardware.
  One deterministic test opens real PseudoRNG with a fake clock and three samples.
- Why/impact: allow explicit multi-device selection and restart-safe drafts
  without leaking selectors or making collection implicit.

## Privacy, filesystem, and diagnostics (2026-08-22)

- Frontend capabilities are only `core:default` and `dialog:default`; general
  opener, filesystem, shell, and logging access is forbidden.
- Frontend errors are stable `SafeError` DTOs. Diagnostics are explicit,
  bounded, redacted in-memory records with generated operation IDs; production
  persistent logging is disabled.
- Entropy, seeds, selectors, serials, OS paths, absolute legacy input paths, and
  arbitrary error chains never cross IPC, preferences, or diagnostics.
- Native and derived artifacts retain containment, no-follow/no-overwrite
  behavior, and backend-known open/reveal targets.
- Why: prevent sensitive material and filesystem authority from leaking across
  the desktop boundary.

## Reports and derived data (2026-08-22)

- XLSX reports use normalized readers for native sessions, read-only RngKitPSG
  v3 BIN/CSV with recorded/estimated timestamp provenance, and validated derived
  bundles. Existing output requires an explicit Cancel/Replace round trip.
- Strict concatenation accepts distinct compatible legacy v3 CSV inputs,
  rejects ambiguous overlap including equal boundaries, revalidates after
  preview, streams creation, records hashes/provenance, and never modifies
  inputs or stores their absolute paths.
- Derived output has distinct directory grammar and a same-stem CSV plus
  schema-versioned manifest, so it cannot be mistaken for a native session.
- Statistical Z and `+/-1.96` are descriptive visual references only: no
  p-values, certification, or pass/fail randomness conclusion.

## Dependencies and delivery (2026-08-22 through 2026-08-23)

- Exact versions live in lockfiles. Node floor is `^20.19.0 || >=22.12.0`, npm
  `>=10`, Rust edition 2024/MSRV 1.85. Dependency upgrades require separate
  validation; prereleases are excluded.
- `rngkit-*` crates use reachable revision `183f3c7811f5593b3b42c2558ac726552b86687d`, never local paths.
- Work follows the approved checkpoint plan. Each checkpoint reports changed
  files, observed validation, manual tests, unrun evidence, limitations, and an
  approval request. Later checkpoints are not implicitly authorized.
- Default tests are deterministic and hardware-free. Physical checks are
  ignored, opt-in, serial, family-scoped, reported per device and OS, and live in
  `src-tauri/tests/hardware.rs`. Absence is unverified, not passed; permission,
  busy, protocol, timeout, and USB failures fail the smoke.
- v1 delivery is an unsigned per-user English NSIS installer with bundled
  offline WebView2. Signing, binary publication, updater, release, and
  deployment require separate approval. Source is public under MIT.

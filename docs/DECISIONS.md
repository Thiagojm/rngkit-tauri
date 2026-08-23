# Decisions

## Product and platform contract (2026-08-22)

- Status: accepted
- Product name is RngKit; v1 is an English Windows 10/11 x64 desktop app.
- Collect, Reports, Combine, and Help share one resizable Tauri 2 window. The
  client-only Svelte 5/Vite/Tailwind CSS 4 frontend uses uPlot; stable mutually
  compatible dependencies are locked, with no prereleases or extra UI framework.
- Why: modernize the familiar RngKitPSG workflows without retaining duplicated
  screens or legacy widget architecture.

## Collection and state contract (2026-08-22)

- Status: accepted
- Exactly one explicitly selected source is used per session. Multiple devices
  remain separate choices; no candidate is selected silently.
- Rust owns the coordinator states (`idle`, `discovering`, `ready`,
  `collecting`, `stopping`, `completed`, `failed`) and rejects stale discovery,
  double starts, stale events, and conflicting file jobs.
- One worker owns the synchronous engine call. Stop and close are cooperative,
  idempotent, and wait for terminal finalization.
- The frontend retains every committed `(sample_index, cumulative_z)` point but
  receives no raw entropy and does not calculate authoritative statistics.
- Why: preserve engine durability and cancellation semantics while keeping UI
  responsiveness and deterministic authority.

## Privacy and filesystem contract (2026-08-22)

- Status: accepted
- Hardware candidates stay in a backend-only transient registry keyed by opaque
  tokens. Serials and OS paths never cross IPC or enter preferences.
- Preferences contain only output root, sample bits, interval, fold, theme, and
  validated window geometry.
- Frontend errors are stable safe DTOs; copied diagnostics are explicit,
  bounded, and redacted. Persistent production logs are disabled in v1.
- Native and derived artifacts preserve containment, no-follow/no-overwrite
  behavior, and backend-known open/reveal targets.
- Why: prevent entropy, selector, path, and diagnostic leakage at the desktop
  boundary.

## Reports and derived data contract (2026-08-22)

- Status: accepted
- XLSX reports are generated only through normalized library readers for native
  sessions, RngKitPSG v3 BIN/CSV, and validated derived bundles.
- Existing XLSX output requires an explicit Cancel/Replace round trip.
- Strict concatenation accepts distinct compatible legacy v3 CSV inputs,
  rejects ambiguous overlap (including equal boundaries), revalidates after
  preview, streams creation, preserves input hashes/provenance, and never
  modifies inputs or stores absolute input paths.
- Derived data uses its own directory grammar and CSV plus schema-versioned
  manifest; it cannot be mistaken for a native collection session.
- Statistical Z and `+/-1.96` remain descriptive visual references with no
  p-values, certification, or pass/fail randomness conclusion.

## Scaffold stack (2026-08-22)

- Status: accepted
- Because `create-tauri-app`'s `svelte-ts` template is SvelteKit, the client-only
  product combines its Tauri 2 Rust/config/icons with the official Vite
  `svelte-ts` SPA layout. Exact versions remain in the lockfiles and context.
- Tailwind `@theme` registers the light-default token namespace. System-dark
  values use a plain CSS media override so the condition survives compilation.
- Browser tests exercise production assets through installed Edge without real
  IPC or hardware; scripts retain the approved names in `AGENTS.md`.
- Frontend capabilities are `core:default` and `dialog:default`. Opener,
  filesystem, shell, and logging permissions are not granted. The official
  template's `tauri-plugin-opener` was not kept.
- `rngkit-*` crates pin git `183f3c7811f5593b3b42c2558ac726552b86687d`.
- Node floor is `^20.19.0 || >=22.12.0`; npm `>=10` (verified on Node 24.18.0 /
  npm 11.16.0). Dependency upgrades require separate validation.
- Why/impact: match the approved SPA and keep the frontend away from general
  filesystem or hardware APIs while later checkpoints add product IPC.

## Application shell (2026-08-22)

- Status: accepted
- A persistent rail exposes the four destinations; the top bar shows product,
  operation status, and a light/dark/system theme control.
- Theme is `data-theme` on `html`. `@theme` keeps light defaults; dark CSS
  variables apply for `data-theme="dark"` and for system dark unless light is
  forced. Collect stacks via a container query so the 800px minimum window does
  not use a side-by-side configuration column.
- Start and Stop never share one surface; disabled controls show a reason. Mock
  snapshots remain for browser tests, and the scenario switch compiles only in
  development. The shell and these rules remain when discovery is wired.

## Coordinator and IPC seam (2026-08-23)

- Status: accepted
- Rust owns collection and file-job transitions. Prohibited transitions return
  stable `SafeError` DTOs. Session IDs and event sequences are coordinator
  state. Tagged camel-case DTOs are independent of `rngkit-*` types.
- Production IPC is `get_app_state`, `refresh_sources`, and `select_source`.
  `apply_dev_scenario` is compiled only under `debug_assertions`. Browser tests
  keep mock snapshots when Tauri is absent.
- Diagnostics are redacted, bounded, in-memory records. They never serialize
  entropy, seeds, selectors, serials, OS paths, or arbitrary error chains.
  Safe-error construction exposes only static canonical messages and generated
  operation IDs. Fold is accepted only for a selected BitBabbler candidate.
- Why: authority and the safe frontend contract must exist before discovery.
- Impact: Discovery calls `discover()` through this coordinator. Do not add
  filesystem, opener, or logging capabilities.

## Discovery and selection (2026-08-23)

- Status: accepted
- `refresh_sources` runs `rngkit_sources::discover()` in Tauri's blocking
  context. Candidates are stored behind random opaque tokens for one
  generation. Frontend DTOs carry token, source id, safe label, variant,
  ordinal, and fold requirement only.
- Default tests inject `FakeDiscovery` and never enumerate or open hardware.
  Refresh invalidates previous tokens and the previous selection. Partial
  family failures become nonblocking safe warnings. Serials and OS paths do
  not cross IPC.
- Discovery never constructs a source or acquires entropy. With PseudoRNG
  compiled in, the candidate represents capability; OS entropy availability is
  authoritative only at explicit `open()` in Checkpoint 8.
- Rejected refresh and selection commands reconcile through `get_app_state`,
  restore usable controls, and surface only structured safe IPC messages.
- Why: explicit multi-device selection without leaking selectors.
- Impact: Checkpoint 7 may persist output root and other safe preferences.
  Do not open a source or collect entropy until Checkpoint 8. The app pins the
  reachable entropy-free discovery revision `183f3c7`.

## Delivery and execution contract (2026-08-22)

- Status: accepted
- Implementation follows the approved checkpoint plan. Each checkpoint ends
  with exact changed files, observed validation, manual test instructions,
  unrun evidence, later limitations, and an approval request.
- Later checkpoints are not implicitly authorized. Material contract changes
  return to design review.
- Default tests are deterministic and hardware-free; physical checks are
  ignored, opt-in, serial, and reported per device and OS.
- v1 delivery is an unsigned per-user English NSIS installer with bundled
  offline WebView2. Signing, release, publication of binaries, updater work,
  and deployment require separate approval.
- Repository source is public under the MIT License.

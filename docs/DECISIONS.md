# Decisions

## Product and platform contract (2026-08-22)

- Status: accepted
- Product name is RngKit; v1 is an English Windows 10/11 x64 desktop app.
- Primary destinations are Collect, Reports, Combine, and Help in one resizable
  window. Collection and live monitoring share one workflow.
- Planned frontend is client-only Svelte 5 with TypeScript, Vite, Tailwind CSS
  4, and uPlot. Tauri 2 provides the native shell.
- Stable mutually compatible dependencies are selected and locked at scaffold
  time; prereleases and an additional component framework are excluded.
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

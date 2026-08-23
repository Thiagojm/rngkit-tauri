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

## Scaffold stack (2026-08-22)

- Status: accepted
- The official `create-tauri-app` `svelte-ts` template is SvelteKit. The
  approved product is a client-only Svelte 5 + Vite SPA, so the scaffold uses
  that template's Tauri 2 Rust/config/icons and the official Vite `svelte-ts`
  frontend layout (`index.html`, `src/main.ts`, `src/App.svelte`).
- Locked at scaffolding: Tauri 2.11.5, `tauri-build` 2.6.3 (latest crates.io
  `tauri-build` cargo resolved), `@tauri-apps/cli` 2.11.4, `@tauri-apps/api`
  2.11.1, Svelte 5.56.10, Vite 8.2.2, TypeScript 6.0.3 (Vite's compatible 6.x
  line, not TypeScript 7), Tailwind CSS 4.3.3 + `@tailwindcss/vite`, uPlot
  1.6.32, Playwright 1.62.1, `tauri-plugin-dialog` 2.7.2.
- Tailwind `@theme` registers the light-default token namespace. System-dark
  values use a plain CSS media override so the condition survives compilation.
- Validation scripts use the approved `format:check`, `test:unit`, and
  `test:e2e` names; browser tests exercise production assets without real IPC
  or hardware and use installed Edge on Windows.
- Frontend capabilities are `core:default` and `dialog:default`. Opener,
  filesystem, shell, and logging permissions are not granted. The official
  template's `tauri-plugin-opener` was not kept.
- `rngkit-*` crates pin git `3f327e9e88679c26683323f116cd6d7b3ea64fff`.
- Node floor is `^20.19.0 || >=22.12.0` (Vite 8); npm `>=10`. Verified on
  Node 24.18.0 / npm 11.16.0.
- Why: match the approved client-only SPA, lock stable versions, and keep the
  frontend from reaching general filesystem or hardware APIs.
- Impact: later checkpoints add pages and IPC on this foundation; dependency
  upgrades need their own validation.

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

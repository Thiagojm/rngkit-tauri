# RngKit

RngKit is a Windows-first desktop application for collecting entropy samples
from explicitly selected hardware or pseudo-random sources, recording native
sessions, monitoring descriptive cumulative statistics, creating XLSX reports,
and safely combining compatible current and RngKitPSG v3 CSV files.

## Status

The four-destination shell is connected to a Rust coordinator through
discovery, selection, session-draft, preference, and collection commands.
Startup restores safe settings, prepares `Documents/RngKit` when no valid saved
output root exists, and displays 2048-bit new-user defaults. After frontend
hydration, one asynchronous discovery runs without opening or selecting a
source; manual Refresh remains available. Default tests inject fake discovery
and fake sources. Safe settings survive restart. Start opens the
selected source, collects until cooperative Stop, and records a native
BIN/CSV/manifest bundle. Open session folder uses a backend-known path.
Closing while collecting confirms Keep collecting or Stop and exit. Debug
builds include a scenario switch that calls `apply_dev_scenario`; production
omits that command and the switch. The live chart retains every committed
descriptive cumulative Z point. Copied diagnostics are bounded and redacted.
Reports inspect a native session directory, a current or legacy v3 BIN/CSV
file, or a derived concatenation bundle and generate a same-stem XLSX with an
explicit Cancel/Replace round trip. Combine accumulates compatible current,
legacy, or mixed CSV files across folders and creates a provenance-bearing
derived bundle without modifying inputs.
Production capabilities stay `core:default` and `dialog:default` with a
restricted CSP. Open commands use backend-known paths only.

Ignored BitBabbler, TrueRNG, RDSEED, and unified discovery smokes live in
`src-tauri/tests/hardware.rs`; default tests do not run them.

The product and installer baseline is `061f66a`; the Checkpoint 18 audit is
complete in the current tree and awaiting user approval. Uninstall and other
unverified evidence are listed in `docs/PROJECT_CONTEXT.md`. The reusable library is
[rngkit-core](https://github.com/Thiagojm/rngkit-core) at
`2cdf311dd206cb5e7320ee520ef1e7a5139cc146`.

A follow-on workflow-improvements design and six-phase implementation plan
dated 2026-08-24 are approved. Phase 1 is complete in `rngkit-core`, and Phase
2 is complete and validated automatically in this app. Phase 3 is complete and
published: the Collect chart has one `Fit all` action and the instrument-style
layout. Phase 4 is complete and published: Reports has one `Choose input`
action for bundles and standalone CSV/BIN inputs, with automated and
browser-integrated validation complete. Phase 5 is complete and published with
automated and browser-integrated validation complete. Phase 6 task-oriented
Help is implemented in the current worktree and its deterministic/browser
validation is complete; native integrated workflow validation remains the
active user gate. Phase 6 approval, commit, and push are separate decisions.

## Sources of truth

- Product contract: `docs/specs/2026-08-22-rngkit-tauri-design.md`
- Execution plan: `docs/plans/2026-08-22-rngkit-tauri-plan.md`
- Approved improvements: `docs/specs/2026-08-24-rngkit-workflow-improvements-design.md`
- Approved phased improvements plan: `docs/plans/2026-08-24-rngkit-workflow-improvements-plan.md`
- Current state: `docs/PROJECT_CONTEXT.md`
- Durable decisions: `docs/DECISIONS.md`
- Roadmap: `TODO.md`

## Stack

Exact versions are locked in `package-lock.json` and `src-tauri/Cargo.lock`.

- Tauri 2.11.5, `@tauri-apps/cli` 2.11.4, `@tauri-apps/api` 2.11.1
- Svelte 5.56.10, Vite 8.2.2, TypeScript 6.0.3
- Tailwind CSS 4.3.3 via `@tailwindcss/vite`
- uPlot 1.6.32
- Playwright 1.62.1 for browser-level scaffold and later mocked-IPC tests
- Rust edition 2024, MSRV 1.85
- Node.js `^20.19.0 || >=22.12.0`, npm `>=10`

## Development

```text
npm ci
npm run tauri dev
```

Frontend-only checks:

```text
npm run check
npm run lint
npm run format:check
npm run test:unit -- --run
npm run test:e2e
npm run build
```

`.github/workflows/ci.yml` runs locked frontend and Rust checks on Windows and
Ubuntu, then `npm run tauri -- build --no-bundle -- --locked`. It does not run ignored
physical tests or build an installer. Observed remote success for `061f66a`:
https://github.com/Thiagojm/rngkit-tauri/actions/runs/32755861549

## Packaging

v1 ships an unsigned per-user English NSIS installer with an embedded offline
WebView2 installer (about 127 MB extra). Signing, SmartScreen, publication,
and updater setup are out of scope. Uninstall must leave user session output
intact.

```text
npm run tauri -- build --bundles nsis -- --locked
```

Local 2026-08-24 evidence (unsigned, not published):
`src-tauri/target/release/bundle/nsis/RngKit_0.1.0_x64-setup.exe`
208.4 MiB, SHA-256 `612BC8F006FA974AE961DDDB4348CE29E8ACBFB7758EF7A7683D6F8B8DDE8DE7`.
The file is not tracked. The user reported offline installation, first launch,
and basic app functionality on Windows. Uninstall and session-data preservation
remain unverified. Windows may warn because the package is unsigned.

## License

MIT. See `LICENSE`.

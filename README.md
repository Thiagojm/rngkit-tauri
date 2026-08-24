# RngKit

RngKit is a Windows-first desktop application for collecting entropy samples
from explicitly selected hardware or pseudo-random sources, recording native
sessions, monitoring descriptive cumulative statistics, creating XLSX reports,
and safely combining compatible RngKitPSG v3 CSV files.

## Status

The four-destination shell is connected to a Rust coordinator through
discovery, selection, session-draft, preference, and collection commands.
Default startup is idle and does not enumerate hardware or restore a source.
Refresh runs real discovery in the native window; default tests inject fake
discovery and fake sources. Safe settings survive restart. Start opens the
selected source, collects until cooperative Stop, and records a native
BIN/CSV/manifest bundle. Open session folder uses a backend-known path.
Closing while collecting confirms Keep collecting or Stop and exit. Debug
builds include a scenario switch that calls `apply_dev_scenario`; production
omits that command and the switch. The live chart retains every committed
descriptive cumulative Z point. Copied diagnostics are bounded and redacted.
Reports inspect a native session directory, a legacy v3 BIN/CSV file, or a
derived concatenation bundle and generate a same-stem XLSX with an explicit
Cancel/Replace round trip. Combine previews compatible legacy v3 CSV files and
creates a provenance-bearing derived bundle without modifying inputs.
Production capabilities stay `core:default` and `dialog:default` with a
restricted CSP. Open commands use backend-known paths only.

Ignored BitBabbler, TrueRNG, RDSEED, and unified discovery smokes live in
`src-tauri/tests/hardware.rs`; default tests do not run them.

Implementation proceeds one independently testable checkpoint at a time and
stops for user approval between checkpoints. The reusable library is
[rngkit-core](https://github.com/Thiagojm/rngkit-core) at
`183f3c7811f5593b3b42c2558ac726552b86687d`.

## Sources of truth

- Product contract: `docs/specs/2026-08-22-rngkit-tauri-design.md`
- Execution plan: `docs/plans/2026-08-22-rngkit-tauri-plan.md`
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
Ubuntu, then `npm run tauri -- build --no-bundle`. It does not run ignored
physical tests or build an installer. Observed remote success:
https://github.com/Thiagojm/rngkit-tauri/actions/runs/32750338632

## License

MIT. See `LICENSE`.

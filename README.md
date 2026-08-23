# RngKit

RngKit is a Windows-first desktop application for collecting entropy samples
from explicitly selected hardware or pseudo-random sources, recording native
sessions, monitoring descriptive cumulative statistics, creating XLSX reports,
and safely combining compatible RngKitPSG v3 CSV files.

## Status

The four-destination shell is connected to a Rust coordinator through
`get_app_state`, `refresh_sources`, and `select_source`. Default startup is
idle and does not enumerate hardware. Refresh runs real discovery in the
native window; default tests inject a fake discovery service. Collection is
not connected yet. Debug builds include a scenario switch that calls
`apply_dev_scenario`; production omits that command and the switch.

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

## License

MIT. See `LICENSE`.

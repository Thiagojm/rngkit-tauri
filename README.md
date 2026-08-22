# RngKit

RngKit is a planned Windows-first desktop application for collecting entropy
samples from explicitly selected hardware or pseudo-random sources, recording
native sessions, monitoring descriptive cumulative statistics, creating XLSX
reports, and safely combining compatible RngKitPSG v3 CSV files.

## Status

This repository is currently planning-only. It contains an approved design,
an approved checkpointed implementation plan, and repository-native context.
There is no application scaffold or executable yet.

Implementation proceeds one independently testable checkpoint at a time and
stops for user approval between checkpoints. The reusable library prerequisite
lives in [rngkit-core](https://github.com/Thiagojm/rngkit-core); its bundle
creation and derived XLSX checkpoint must be completed before app scaffolding.

## Sources of truth

- Product contract: `docs/specs/2026-08-22-rngkit-tauri-design.md`
- Execution plan: `docs/plans/2026-08-22-rngkit-tauri-plan.md`
- Current state: `docs/PROJECT_CONTEXT.md`
- Durable decisions: `docs/DECISIONS.md`
- Roadmap: `TODO.md`

## Planned stack

Tauri 2, Svelte 5, TypeScript, Vite, Tailwind CSS 4, uPlot, and the versioned
`rngkit-core` Rust crates. Exact stable versions will be selected and locked
when the scaffold checkpoint is authorized.

## License

MIT. See `LICENSE`.

# TODO

## Done

- Approved Windows-first design/plan and `rngkit-core` Gate A at `2cdf311`.
- Original v1 checkpoints 3–18: locked Tauri/Svelte shell, coordinator,
  collection, chart, close policy, reports, Combine, security/a11y, CI, and
  unsigned NSIS baseline. Checkpoint 18 evidence and limitations are recorded
  in `docs/PROJECT_CONTEXT.md`.
- Approved 2026-08-24 workflow-improvements design and six-phase plan.
- Phase 1: generalized standalone readers and CSV concatenation in
  `rngkit-core`, published at the exact reachable revision.
- Phase 2: default root, 2048-bit defaults, asynchronous discovery, and safe
  recovery behavior, automatically validated.
- Phase 3: instrument-style chart, one `Fit all` action, and following/viewport
  race fix, automatically and browser-integrated validated.
- Phase 4: unified Reports chooser and standalone current/legacy CSV/BIN
  reports, automatically and browser-integrated validated.
- Phase 5: incremental mixed-format Combine, opaque-ID Remove/Clear, and
  schema-2 derived bundles, automatically and browser-integrated validated and
  published.
- Phase 6 implementation: task-oriented Help in approved order, production-copy
  audit, focused unit/E2E regression updates, and complete deterministic suite.

## In progress

- Phase 6 user-validation gate: native integrated workflows remain unverified.

## Next steps

1. Run the Phase 6 native manual validation from the approved plan.
2. Obtain explicit Phase 6 approval before any commit or push authorization.

## Backlog

- NSIS uninstall and session-data preservation; signing, releases, updater,
  Store, Linux packaging, and deployment remain outside current authority.
- Native hardware Collect, unplug-during-read, other devices/folds, and Linux
  physical behavior.
- Native 100k/1M chart interaction; native Reports/Combine dialogs; Windows
  scaling and screen-reader sampling; symlink inspect privilege coverage.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain non-goals or future work.

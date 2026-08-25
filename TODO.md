# TODO

## Done

- Approved Windows-first design/plan and `rngkit-core` Gate A at `7c79814`.
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
- Phase 6: task-oriented Help in approved order, production-copy audit, focused
  unit/E2E regressions, and complete deterministic suite reviewed, corrected,
  and published.
- 2026-08-25 Phase 2: flat legacy concatenation Reports integration, explicit
  chart-axis/source context retention, exact library pin, and locked app
  validation completed and published at app commit `b946c4d`.
- 2026-08-25 Phase 3: transient typed artifact outcome notices, acknowledgement,
  backend-known working-folder commands, safe artifact path allowlisting, and
  locked app validation completed and published at app commit `44e0d65`.
- 2026-08-25 Phase 4: one-shot outcome dialog, backend-approved artifact
  actions, contextual working-folder buttons, and complete frontend/browser
  validation completed and published at app commit `b137419`.
- Terminal collection outcome reconciliation and local-clock XLSX labels are
  integrated against `rngkit-core` revision `495c3f5`, with complete frontend,
  locked Rust, Rust 1.85, E2E, build, and no-bundle release validation passed.

## In progress

- 2026-08-25 Phase 5 user-validation gate: Help/documentation now describe the
  current artifact, path, chart-axis, and recovery contracts; deterministic
  and native acceptance evidence is separated, with native integrated workflow
  validation still required.

## Next steps

1. Run the complete native acceptance checklist from the approved plan.
2. Separately authorize any Phase 5 commit/push, release, or publication work.

## Backlog

- NSIS uninstall and session-data preservation; signing, releases, updater,
  Store, Linux packaging, and deployment remain outside current authority.
- Native hardware Collect, unplug-during-read, other devices/folds, and Linux
  physical behavior.
- Native 100k/1M chart interaction; native Reports/Combine dialogs; Windows
  scaling and screen-reader sampling; symlink inspect privilege coverage.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain non-goals or future work.

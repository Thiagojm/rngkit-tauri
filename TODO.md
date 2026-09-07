# TODO

## Done

- Approved Windows-first design/plan and `rngkit-core` Gate A at `7c79814`.
- Original v1 and six workflow-improvement phases: Tauri/Svelte shell,
  collection/chart/close policy, unified standalone Reports, mixed CSV Combine,
  task-oriented Help, security/accessibility, CI, and unsigned NSIS baseline.
- Artifact feedback and report-chart phases: flat legacy concatenation,
  contextual charts, typed one-shot outcomes, backend-known folder actions,
  and complete frontend/browser validation.
- Terminal collection outcomes display immediately; recorded charts use local
  clock context; selected `.bin` basenames remain authoritative; user-visible
  Windows paths omit `\\?\`. The app pins reachable core revision `3dc969d`.
- Complete frontend, locked Rust, Rust 1.85, E2E, build, and no-bundle release
  validation passed. Native PseudoRNG Collect and manifest-backed Reports smoke
  validation passed, including immediate outcome and normalized full paths.
- Help, diagnostics, README, and revision tests now track the exact locked
  `rngkit-core` revision from Cargo sources.
- Corrected Combine outcome-path assertions behind CI run `33981573132`:
  canonical component containment and privacy checks outside the allowed outcome.
  Remote Ubuntu tests passed in run `34002210623`; its subsequent Clippy
  failure is corrected by gating the Windows-only test import. Windows exposed
  the same path-alias assertion in Security; Security/Collection now canonicalize
  outcome paths too. New CI pending.

## In progress

- Remaining native acceptance variants are tracked below.

## Next steps

1. Validate flat/standalone legacy/current and BIN-only report variants,
   selected-BIN chart title, mixed cross-folder Combine, artifact open actions,
   Help/theme/keyboard/minimum-window behavior, and Excel chart rendering.

## Backlog

- NSIS uninstall and session-data preservation; signing, releases, updater,
  Store, Linux packaging, and deployment remain outside current authority.
- Native hardware Collect, unplug-during-read, other devices/folds, and Linux
  physical behavior.
- Native 100k/1M chart interaction; native Reports/Combine dialogs; Windows
  scaling and screen-reader sampling; symlink inspect privilege coverage.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain non-goals or future work.

## Session validation follow-up (2026-09-06)

- Implemented Start-time numeric validation with explanatory dialogs and moved
  terminal session actions into Session. Focused regression tests passed.
- Next: native acceptance of invalid bits/interval, correction and Start, and
  terminal action placement. No hardware collection is required for this check.

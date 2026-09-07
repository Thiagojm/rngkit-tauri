# TODO

## Done

- Windows-first v1 and six workflow phases: Collect/chart/close policy, standalone
  Reports, mixed CSV Combine, Help, accessibility, CI and unsigned NSIS baseline.
- Artifact outcomes, local-clock report charts, selected-BIN basename and Windows
  path display fixes; exact rngkit-core revision `3dc969d` remains pinned.
- Historical full frontend/Rust/MSRV/no-bundle validation and native PseudoRNG
  Collect/manifest Reports smoke passed; detailed evidence is in PROJECT_CONTEXT.
- Cross-platform outcome-path CI repair passed (`34002608469`); numeric-validation
  CI passed (`34072410813`). Numeric rules and Session actions are user-validated.
- Compact Monitoring preserves chart size; browser checks and user visual approval
  passed. Layout CI run `34077305701` is in progress at this maintenance pass.

## Next steps

1. Design a Help copy/layout refresh: current numeric rules, Session actions,
   clearer report bundle requirements, less jargon, and readable visual hierarchy.
   The user requested assessment; design and implementation are not yet approved.
2. Confirm completion of the Collect layout CI run.
3. Validate standalone legacy/current and BIN-only Reports, selected-BIN chart title,
   cross-folder mixed Combine/derived reports, artifact actions and Excel charts.
4. Complete native Help/theme/keyboard/minimum-window and scaling acceptance.

## Backlog

- Native hardware collection, unplug-during-read, other devices/folds and Linux
  physical behavior; opt-in only.
- Native 100k/1M chart interaction, Reports/Combine dialogs, screen-reader sampling
  and symlink inspection privilege coverage.
- NSIS uninstall/session-data preservation, signing, releases, updater, Store,
  Linux packaging and deployment; separate authorization remains required.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume and statistical
  inference remain non-goals or future work.

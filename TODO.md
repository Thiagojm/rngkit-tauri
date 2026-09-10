# TODO

## Done

- Windows-first v1 and six workflow phases: Collect/chart/close policy, standalone
  Reports, mixed CSV Combine, Help, accessibility, CI and unsigned NSIS baseline.
- Artifact outcomes, local-clock report charts, selected-BIN basename and Windows
  path display fixes; exact rngkit-core revision `4e0e43d` is pinned.
- Historical full frontend/Rust/MSRV/no-bundle validation and native PseudoRNG
  Collect/manifest Reports smoke passed; detailed evidence is in PROJECT_CONTEXT.
- Cross-platform outcome-path CI repair passed (`34002608469`); numeric-validation
  CI passed (`34072410813`). Numeric rules and Session actions are user-validated.
- Compact Monitoring preserves chart size; browser checks and user visual approval
  passed. Layout CI run `34077305701` passed (verified 2026-09-09).

- User-reported manual acceptance (2026-09-07): refreshed Help and Reports/Combine,
  including standalone current/legacy CSV/BIN, cross-folder combinations and XLSX
  titles, timestamps and charts in Excel.
- Device setup Phases 1–3: Help disclosures, Collect navigation, reviewed kit
  and Linux helper, and Help opens `device-setup` through
  `open_device_setup_folder`. User-reported native acceptance on 2026-09-07
  covers guidance, kit review, and folder-open.
- Review fixes: explicit Bash commands, retryable udev reload, and exclusion of
  TrueRNG INF/CAT pending documented redistribution permission.
- CI run `34144732140` SC1007 fix: explicit empty CDPATH in both Bash scripts.
  Local syntax/helper checks and remote run `34147766895` at `ad5b35f` passed.
- Corrected unsigned NSIS rebuilt/inspected 2026-09-09: ten matching kit files,
  no TrueRNG INF/CAT, offline WebView2 embedded. Non-elevated update and native
  Help folder-open passed with network connected; old INF/CAT copies remain.

## Next steps

- [x] Integrate trng3-rs `08f1889` through reachable rngkit-core `56c0c84`.
- [x] User reported successful native TrueRNG fix acceptance on 2026-09-10
  using the rebuilt executable; exact latency and saved count were not supplied.
- [x] Review source freshness: normal BitBabbler/RDSEED paths need no serial-style
  purge. Evidence and two BitBabbler error-path findings: `docs/source-freshness-review.md`.
- [x] Fix BitBabbler reuse after failed I/O and bound sync/purge locally; regressions pass.
- [x] Publish BitBabbler `e4cc6c6` and integrate through core `4e0e43d`.
- [ ] Validate native BitBabbler acquisition, folds and disconnect behavior.

1. Validate installation and Help kit access with network disconnected as a
   standard user; clean-machine WebView2 remains untested. Decide cleanup of old
   TrueRNG INF/CAT retained by updates. Redistribution permission is required
   only before including them again. Setup must not run implicitly.
2. Separately authorize Ubuntu/Debian hardware validation (both devices after
   login/reconnect, repeated setup, unrelated FTDI unchanged) when a host exists.
3. Validate remaining artifact open/folder actions and native scaling acceptance.

## Backlog

- Native hardware collection, unplug-during-read, other devices/folds and Linux
  physical behavior; opt-in only.
- Native 100k/1M chart interaction, Reports/Combine dialogs, screen-reader sampling
  and symlink inspection privilege coverage.
- NSIS uninstall/session-data preservation, signing, releases, updater, Store,
  Linux packaging and deployment; separate authorization remains required.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume and statistical
  inference remain non-goals or future work.

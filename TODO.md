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

- User-reported manual acceptance (2026-09-07): refreshed Help and Reports/Combine,
  including standalone current/legacy CSV/BIN, cross-folder combinations and XLSX
  titles, timestamps and charts in Excel.
- Device setup Phases 1–3: Help disclosures, Collect navigation, reviewed kit
  and Linux helper, and Help opens `device-setup` through
  `open_device_setup_folder`. User-reported native acceptance on 2026-09-07
  covers guidance, kit review, and folder-open. Unsigned NSIS includes the kit
  with matching hashes; installed-app access is not established.

## Next steps

1. Separately authorize installing the unsigned NSIS and opening the kit from
   Help offline, as a standard user. Setup must not run implicitly.
2. Separately authorize Ubuntu/Debian hardware validation (both devices after
   login/reconnect, repeated setup, unrelated FTDI unchanged) when a host exists.
3. Confirm completion of the Collect layout CI run.
4. Validate remaining artifact open/folder actions and native scaling acceptance.

## Backlog

- Native hardware collection, unplug-during-read, other devices/folds and Linux
  physical behavior; opt-in only.
- Native 100k/1M chart interaction, Reports/Combine dialogs, screen-reader sampling
  and symlink inspection privilege coverage.
- NSIS uninstall/session-data preservation, signing, releases, updater, Store,
  Linux packaging and deployment; separate authorization remains required.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume and statistical
  inference remain non-goals or future work.

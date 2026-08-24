# TODO

## Done

- Approved Windows-first design/plan and `rngkit-core` Gate A at `2cdf311`.
- Checkpoints 3–17: locked Tauri/Svelte shell, coordinator, collection, chart,
  close policy, physical smokes, reports, Combine, security/a11y, CI, unsigned
  NSIS. HEAD `061f66a`.
- Checkpoint 18: design-to-evidence trace, deterministic suite, tracked-file
  scan, context update, and user-reported offline install/basic app smoke. No
  required product work was silently deferred.
- Approved 2026-08-24 workflow-improvements design and six-phase plan.
- Phase 1: generalized standalone readers and CSV concatenation completed and
  published in `rngkit-core`.
- Phase 2: app pin/default-root/2048-bit/automatic-discovery implementation
  completed and validated automatically; recovery warnings and parallel
  security fixtures were corrected during review. Native user test remains
  unverified.

## In progress

- No implementation phase is currently authorized. Do not begin Phase 3
  without a new explicit request.

## Next steps

1. Run the Phase 2 native manual validation from the approved plan.
2. Begin Phase 3 only after explicit authorization.

## Backlog

- NSIS uninstall and session-data preservation.
- Native hardware Collect UI, unplug-during-read, other folds/devices, Linux
  physical behavior.
- Native uPlot render/interaction at 100k/1M points; native Reports/Combine
  dialogs; Windows 100%/150%/200% scaling; screen-reader sampling; Windows
  file-symlink inspect (privilege 1314).
- Signing, binary releases, updater, Store, Linux packaging, and deployment
  remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and
  statistical inference remain non-goals or future work.

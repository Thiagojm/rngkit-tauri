# TODO

## Done

- Approved Windows-first design/plan and `rngkit-core` Checkpoints 1–2 plus Gate A at revision `183f3c7`.
- Checkpoints 3–7: locked Tauri/Svelte/Vite/Tailwind shell, coordinator DTOs,
  fake discovery, and atomic preferences; default start does not enumerate hardware.
- Checkpoints 8–10: collection worker, live uPlot chart, close interception,
  Stop and exit, and redacted Copy diagnostics.
- Checkpoint 11: ignored serial Windows smokes passed one BitBabbler White
  fold-0, one TrueRNG, and one RDSEED (ordinal 1, 3 fake-clock samples, native
  bundle ok); unified discovery also listed PseudoRNG without opening. Other
  folds/devices, unplug behavior, hardware UI collection, and Linux physical
  behavior remain unverified.
- Checkpoints 12–13: native and legacy v3 report inspection/generation.
  Completed/interrupted native and CSV-only, BIN-only, or paired legacy inputs
  write same-stem XLSX; invalid inputs fail safely, legacy files remain
  unchanged, existing output needs explicit Replace, and artifact opening is
  idle-only. Native/legacy desktop walkthroughs remain unverified.
- Checkpoint 14: Combine preview/create and derived reports. Compatible legacy
  v3 CSVs preview chronologically and write a no-overwrite derived CSV/manifest bundle; per-input errors, incompatible,
  overlapping, changed-after-preview, and write failures leave inputs unchanged
  and no partial bundle. Derived folders inspect and generate contained XLSX.
  Native Combine/Reports desktop walkthroughs remain unverified.

## In progress

- Checkpoint 14 user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 14, then authorize Checkpoint 15 (hardening).

## Backlog

- Checkpoints 15–18: hardening, CI, installer, and final audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

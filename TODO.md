# TODO

## Done

- Approved Windows-first RngKit design and staged implementation plan.
- `rngkit-core` Checkpoints 1–2 and Gate A at revision `183f3c7`.
- Checkpoints 3–7: locked Tauri/Svelte/Vite/Tailwind shell, coordinator DTOs,
  fake-injected discovery, and atomic preferences. Default start does not
  enumerate hardware.
- Checkpoints 8–10: collection worker, live uPlot chart, close interception,
  Stop and exit, and redacted Copy diagnostics.
- Checkpoint 11: ignored serial Windows smokes passed one BitBabbler White
  fold-0, one TrueRNG, and one RDSEED (ordinal 1, 3 fake-clock samples, native
  bundle ok); unified discovery also listed PseudoRNG without opening. Other
  folds/devices, unplug behavior, hardware UI collection, and Linux physical
  behavior remain unverified.
- Checkpoints 12–13: native and legacy v3 report inspection/generation.
  Completed/interrupted native sessions and CSV-only, BIN-only, or paired legacy
  inputs write same-stem XLSX; invalid inputs fail safely, legacy files remain
  unchanged, existing output needs explicit Replace, and artifact opening is
  idle-only. Native/legacy desktop walkthroughs remain unverified; derived
  inputs and Combine remain unexposed.

## In progress

- Checkpoint 13 user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 13, then authorize Checkpoint 14 (Combine and derived reports).

## Backlog

- Checkpoints 14–18: Combine/derived reports, hardening, CI, installer, and final
  audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

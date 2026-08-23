# TODO

## Done

- Approved Windows-first RngKit design and staged implementation plan.
- `rngkit-core` Checkpoints 1–2 and Gate A at revision `183f3c7`.
- Checkpoints 3–7: locked Tauri/Svelte/Vite/Tailwind shell, coordinator DTOs,
  fake-injected discovery, and atomic preferences. Default start does not
  enumerate hardware.
- Checkpoints 8–10: collection worker, live uPlot chart, close interception,
  Stop and exit, and redacted Copy diagnostics.
- Checkpoint 11A: ignored serial BitBabbler app smoke. This Windows host ran
  fold-0 for one White ordinal-1 device (3 samples, fake clock, native bundle
  ok). Other folds, extra devices, TrueRNG, RDSEED, and the Collect UI path
  remain unverified.

## In progress

- Checkpoint 11A user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 11A, then authorize Checkpoint 11B (TrueRNG app
   integration) only if a device is available; otherwise record unverified.

## Backlog

- Checkpoints 11B–18: remaining physical families, reports, Combine,
  hardening, CI, installer, and final audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

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
  ok). Other folds, extra BitBabbler devices, and the Collect UI path remain
  unverified.
- Checkpoint 11B: ignored serial TrueRNG app smoke. This Windows host ran one
  ordinal-1 device (variant TrueRNG, 3 samples, fake clock, native bundle ok).
  Unplug-during-read, extra ports, RDSEED, and the Collect UI path remain
  unverified.

## In progress

- Checkpoint 11B user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 11B, then authorize Checkpoint 11C (RDSEED and unified
   discovery) only if the host can report support; otherwise record unverified.

## Backlog

- Checkpoints 11C–18: RDSEED, reports, Combine,
  hardening, CI, installer, and final audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

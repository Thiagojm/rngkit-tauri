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
  Unplug-during-read, extra ports, and the Collect UI path remain unverified.
- Checkpoint 11C: ignored RDSEED and unified discovery smokes. This Windows
  host ran RDSEED ordinal 1 (variant RDSEED, 3 samples, fake clock, native
  bundle ok) and listed BitBabbler, TrueRNG, RDSEED, and PseudoRNG without
  opening. Collect UI on hardware remains unverified.

## In progress

- Checkpoint 11C user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 11C, then authorize Checkpoint 12 (native report
   inspection and generation).

## Backlog

- Checkpoints 12–18: reports, Combine, hardening, CI, installer, and final
  audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

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
- Checkpoint 12: native report inspection and generation. Completed and
  interrupted committed-prefix sessions inspect; live/corrupt/unsupported
  inputs fail safely; existing XLSX needs explicit Replace; artifact opening is
  idle-only. Legacy/derived inputs and the native desktop workbook walkthrough
  remain unverified.

## In progress

- Checkpoint 12 user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 12, then authorize Checkpoint 13 (legacy v3 reports).

## Backlog

- Checkpoints 13–18: legacy/derived reports, Combine, hardening, CI, installer, and final
  audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

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
- Checkpoints 14–16: Combine/derived reports, security/a11y, and locked
  Windows/Ubuntu CI (observed `a9f99e5` at https://github.com/Thiagojm/rngkit-tauri/actions/runs/32750338632).
- Checkpoint 17: unsigned per-user English NSIS with offline WebView2; local
  `RngKit_0.1.0_x64-setup.exe` (208.4 MiB). Install/uninstall remain user-verified.

## In progress

- Checkpoint 17 user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 17 after installer install/launch/uninstall, then authorize Checkpoint 18 (final audit).

## Backlog

- Checkpoint 18: final context and acceptance audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

# TODO

## Done

- Approved Windows-first RngKit design and staged implementation plan.
- `rngkit-core` Checkpoints 1–2 and Gate A at revision `183f3c7`.
- Checkpoints 3–4: locked Tauri/Svelte/Vite/Tailwind foundation, pinned library
  revision, minimal capabilities, browser coverage, four-destination shell,
  semantic controls, theme control, and development-only mock states.
- Checkpoint 5: Rust coordinator, camel-case DTOs, safe errors/diagnostics, fold
  validation, and `get_app_state` without arbitrary production transitions.
- Checkpoint 6: real discovery through `rngkit_sources::discover()`, opaque
  candidate tokens, explicit selection, and a fake discovery service for
  default tests. Sources are not opened; the pinned library advertises the
  compiled PseudoRNG capability without requesting OS entropy. Rejected
  refresh/selection IPC reconciles authoritative state and exposes safe errors.
- Checkpoint 7: atomic schema-versioned preferences, native directory dialog,
  transactional draft, and mixed-DPI-safe physical geometry. Invalid files reset
  completely, Windows replacement is atomic, and selection is never persisted.
- Checkpoints 8–9: collection worker, sequenced metric DTOs with numeric
  cumulative Z, Start/Stop, native PseudoRNG recording, backend-known session
  folder opening, and a live uPlot chart that retains every committed point.
- Checkpoint 10: close interception, Stop and exit after worker finalization,
  idempotent close while stopping, safe error recovery, and redacted Copy
  diagnostics. Channel loss and active reloads reconcile to queryable terminal
  state. Automated frontend/Rust/MSRV suites and rendered browser recovery/copy
  checks pass. Native Tauri fixture validation covered Keep collecting and Stop
  and exit; real-worker close finalization and native clipboard remain unverified.

## In progress

- Checkpoint 10 user approval. No later checkpoint is authorized.

## Next steps

1. Approve Checkpoint 10, then authorize Checkpoint 11A (BitBabbler app
   integration) only if a device is available; otherwise record unverified.

## Backlog

- Checkpoints 11-18: physical validation, reports, Combine,
  hardening, CI, installer, and final audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

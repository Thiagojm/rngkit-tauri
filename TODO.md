# TODO

## Done

- Approved Windows-first RngKit design and staged implementation plan.
- `rngkit-core` Checkpoints 1–2 and Gate A at
  `183f3c7811f5593b3b42c2558ac726552b86687d`.
- Checkpoint 3: locked Tauri 2 / Svelte 5 / Vite / Tailwind CSS 4 foundation
  with pinned library git revision, conditional system-theme tokens, browser
  smoke coverage, and minimal capabilities.
- Checkpoint 4: four-destination shell, semantic controls, theme control, and
  development-only mocked product states.
- Checkpoint 5: Rust coordinator, camel-case DTOs, canonical safe errors,
  bounded redacted diagnostics, source-aware fold validation, and
  `get_app_state`. Production has no arbitrary transition command.
- Checkpoint 6: real discovery through `rngkit_sources::discover()`, opaque
  candidate tokens, explicit selection, and a fake discovery service for
  default tests. Sources are not opened; the pinned library advertises the
  compiled PseudoRNG capability without requesting OS entropy. Rejected
  refresh/selection IPC reconciles authoritative state and exposes safe errors.

## In progress

- None. No implementation checkpoint is currently authorized.

## Next steps

1. Authorize and implement Checkpoint 7: session draft, native dialogs, and
   safe preferences. Do not start collection.

## Backlog

- Checkpoints 7-18: collection, chart, lifecycle,
  physical validation, reports, Combine, hardening, CI, installer, and final
  audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

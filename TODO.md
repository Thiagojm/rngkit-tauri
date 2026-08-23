# TODO

## Done

- Approved Windows-first RngKit design and staged implementation plan.
- `rngkit-core` Checkpoints 1–2 and Gate A at
  `3f327e9e88679c26683323f116cd6d7b3ea64fff`.
- Checkpoint 3: locked Tauri 2 / Svelte 5 / Vite / Tailwind CSS 4 foundation
  with pinned library git revision, conditional system-theme tokens, browser
  smoke coverage, and minimal capabilities.
- Checkpoint 4: four-destination shell, semantic controls, theme control, and
  development-only mocked product states.
- Checkpoint 5: Rust coordinator, camel-case DTOs, canonical safe errors,
  bounded redacted diagnostics, source-aware fold validation, and
  `get_app_state`. Production has no arbitrary transition command.

## In progress

- None. No implementation checkpoint is currently authorized.

## Next steps

1. Authorize and implement Checkpoint 6: real discovery and explicit transient
   selection. Do not open a source or collect entropy.

## Backlog

- Checkpoints 6-18: discovery, collection, chart, lifecycle,
  physical validation, reports, Combine, hardening, CI, installer, and final
  audit.
- Signing, binary releases, updater, Store submission, Linux packaging, and
  deployment remain outside v1 implementation authority.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume, and statistical
  inference remain explicit non-goals or future work.

# Device setup implementation plan

Status: Approved by the user on 2026-09-07. Phases 1–3 complete after user
acceptance, including native folder-open. Unsigned NSIS packaging inspected
on 2026-09-07; installed-app checks and Linux hardware validation remain
unauthorized.
Review corrections (2026-09-07): Linux commands now invoke Bash and every apply
reloads udev for retry recovery. TrueRNG INF/CAT inclusion is blocked pending
documented redistribution permission; the files are excluded and Help identifies
the manufacturer download. Earlier installer evidence predates these corrections;
a rebuild and inspection remain separately authorized work.
Design: `docs/specs/2026-09-07-device-setup-design.md` (approved).

## Execution contract

Inspect the active checkout and read AGENTS.md, PROJECT_CONTEXT, DECISIONS, TODO,
README and the design before editing. Preserve existing documentation changes.
Use locked dependencies and verify device behavior against the pinned core source,
not an unrelated local library checkout. Do not modify either legacy repository.

Completion is delivered in three phases. Each phase must leave a usable app and
end with a report and user-validation gate. Approval of this plan does not authorize
implementation; the subsequent handoff must name the explicitly authorized phase.
Do not implement later-phase placeholders. Commit, push, installer build/execution,
physical tests and publication need separate authorization.

## Phase 1 — Offline guidance and Collect navigation

Goal: users reach accurate device setup instructions without leaving the app.

Files:
- `src/pages/HelpPage.svelte`, `src/pages/HelpPage.test.ts`
- `src/components/collect/SourceDiscovery.svelte` and its existing test
- `src/components/app/AppShell.svelte` and its existing test
- `src/pages/CollectPage.svelte`, `src/state/app-state.svelte.ts` only as needed
  to connect the existing navigation flow; do not create a generic router.
- `tests/e2e/accessibility.spec.ts`, `tests/e2e/scaffold.spec.ts`
- `src/copy.ts` if shared labels belong in the existing copy structure.

Steps:
1. Trace SourceDiscovery through CollectPage/AppShell and the existing destination
   state. Add a narrow callback or existing-state action for Device setup. Focus
   and scroll after Help renders; consume the target so normal Help navigation
   does not unexpectedly jump again. Do not persist this transient intent.
2. Add four native disclosures under Choosing a source, retaining all eight topic
   links. Include exact supported IDs, WinUSB versus CDC distinction, reconnect,
   login/permissions guidance and Refresh sources. Linux manual steps must match
   the approved narrow rules; do not refer to a helper as bundled before phase 2.
3. Include a small Collect link near Refresh sources. Keep it usable during a
   collection and preserve all collection state and Monitoring dimensions.
4. Cover focused navigation, disclosure keyboard behavior and unchanged state in
   existing tests. Browser tests cover both themes, minimum/default windows and
   increased text size. No folder button or new IPC yet.

Verification (Windows uses npm.cmd):
- `npm run test:unit -- --run`
- `npm run check`
- `npm run test:e2e` (includes production build)
- `npm run lint` after E2E, then `npm run format:check`
- `git diff --check`

Gate: report automated/browser evidence and ask the user to test the Collect link,
Help focus/disclosures and return to an active collection in the native app. Do
not claim device installation success from UI tests. Update project context/TODO
and durable decisions for the delivered contract. Stop for phase 2 authorization.

## Phase 2 — Reviewed offline kit and Linux helper

Goal: prepare self-contained resources without registering them in the installer.

Create under `src-tauri/resources/device-setup/`:
- `README.md`, `SOURCES.md`, `THIRD-PARTY-NOTICES.md`
- `windows/bitbabbler/` verified Zadig and required redistribution materials
- `windows/truerng3/` verified INF/CAT
- `linux/setup-rng-devices.sh`
- `linux/60-rngkit-bitbabbler.rules`, `linux/60-rngkit-truerng3.rules`

Add `tests/device-setup/linux-setup.test.sh` for isolated helper checks and extend
`.github/workflows/ci.yml` with an Ubuntu-only check. Do not add a shell framework.
Update phase 1 Help references only once the helper contract is final.

Steps:
1. Obtain files from official upstream URLs; record fixed versions, hashes,
   signature validation and distribution requirements in SOURCES.md. Inspect the
   INF device IDs and preserve the paired catalog. Never execute installers while
   verifying them. If verification/distribution cannot be established, report the
   specific blocked asset and stop that part; do not silently use legacy binaries.
2. Implement the two narrow rules from the design with group rngkit and mode 0660.
   Validate ModemManager matching on USB parents and tty scope. Exclude generic
   FTDI IDs, TrueRNGpro, stty hooks, world access and fixed shared symlinks.
3. Implement this CLI: `--device bitbabbler|truerng3|both --user USER`, plus `--check`
   and `--help`. Require explicit device/user for check and apply. Help/check do
   not require root; apply does. Unknown/missing arguments fail before mutation.
4. Preflight supported OS, user, tools, source rules and destination conflicts.
   Check all selected destinations before creating groups or writing files. Reject
   symlinks/nonregular rule targets; preserve different existing content. Require
   existing identical rules to have safe ownership/mode, or report the mismatch.
5. Apply only missing group/membership/rules, with root ownership and mode 0644.
   Avoid partial rule content through temporary files in the destination directory
   and atomic placement; do not overwrite administrator files. Report completed
   operations if a later command fails; never print unconditional success.
6. Reload udev only after successful changes. Do not trigger all devices, modify
   modules, stop services or install packages. Derive runtime prerequisites from
   the actual Rust link configuration. Explain fresh login and reconnection.
7. Document exact rollback based on reported changes, preserving pre-existing
   membership/groups/rules. Explain that setup checks cannot certify hardware I/O.

Verification:
- `bash -n src-tauri/resources/device-setup/linux/setup-rng-devices.sh`
- `bash tests/device-setup/linux-setup.test.sh`
- Run ShellCheck if available; do not install it silently.
- Test bad arguments, invalid/root user, missing prerequisites, selected-device
  scope, identical rules, conflicting files, symlinks and command failures.
- Execute mutation cases only in a disposable unprivileged sandbox with stubbed
  commands and temporary destinations. Production execution must never accept an
  environment-controlled alternate root while elevated. A root-dependent test
  that could touch host /etc or groups must not run in normal CI.
- Confirm --check produces no writes; inspect rule matching with udev tooling in a
  disposable Linux environment when available. Record unavailable checks honestly.
- Verify source hashes and `git diff --check`; run frontend checks if Help changed.

Gate: user reviews kit instructions and results. Request separately authorized
Ubuntu/Debian hardware validation: both devices after login/reconnect, repeated
setup and unrelated FTDI device unaffected. If no Linux host is available, record
native acceptance pending and require an explicit decision to proceed with that
limitation. Update context/TODO. Stop for phase 3 authorization.

## Phase 3 — Resource folder command and packaging configuration

Goal: Help opens only the installed support directory, with resources packaged.

Files:
- `src-tauri/tauri.conf.json`
- `src-tauri/src/commands/device_setup.rs` (new small command module)
- `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
- `src-tauri/src/reports/mod.rs` only if a minimal visibility adjustment is needed
  to reuse the current folder opener; avoid a new generic opening framework.
- `src/ipc/client.ts`, `src/ipc/client.test.ts`
- `src/pages/HelpPage.svelte`, `src/pages/HelpPage.test.ts`
- Existing browser tests as needed; AGENTS.md and docs for the new IPC contract.

Steps:
1. Register fixed device-setup resources with an explicit destination mapping.
   Check actual Tauri development and packaged resource resolution, including paths
   with spaces. Do not introduce a cwd or arbitrary-path fallback.
2. Implement no-argument open_device_setup_folder. Resolve the fixed directory,
   validate type/containment and unsafe links using existing path-validation
   patterns, and reuse the platform opener. Return existing SafeError shapes.
3. Register in debug and release command lists. Add a typed frontend wrapper with
   the existing safe browser mock behavior. No capability expansion or shell IPC.
4. Add Open device setup folder to Help with accessible pending/error feedback.
   The button opens the folder only and never starts scripts or executables.
5. Add Rust tests using temporary resources and a fake opener for success,
   missing/wrong-type/unsafe paths and opening failure. Add frontend invocation
   and error-feedback coverage; no default tests open folders or devices.

Verification:
- Full frontend unit/check/E2E/lint/format suite as in phase 1.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo check --locked --manifest-path src-tauri/Cargo.toml --all-targets`
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --all-targets`
- `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --doc`
- `cargo +1.85.0 check --locked --manifest-path src-tauri/Cargo.toml --all-targets`
- `cargo +1.85.0 test --locked --manifest-path src-tauri/Cargo.toml --all-targets`
- `git diff --check`

Gate: ask the user to verify folder opening in native development mode. Report
packaging configuration separately from packaged evidence. With separate installer
build authorization, run `npm run tauri -- build --bundles nsis -- --locked` and
inspect included files/hashes. With separate installation authorization, verify
installed Help/folder access offline as a standard user; setup must never run
implicitly. Document any pending installed-app check instead of marking it passed.

## Handoff and review

After plan approval, use tjm-handoff in artifact-backed mode. Cite this plan and the
approved design; state the currently authorized phase and existing documentation
changes. Implementation belongs to the receiving agent. The authoring agent will
review implementation only when asked; a review request does not itself authorize
fixes, further phases or remote actions.

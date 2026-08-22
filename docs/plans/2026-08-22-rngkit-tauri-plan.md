# RngKit Tauri Application Implementation Plan

**Status:** Ready for separately authorized implementation
**Date:** 2026-08-22
**Approved design:** `docs/specs/2026-08-22-rngkit-tauri-design.md`
**Application root:** `D:\Projetos\rustie\rngkit-tauri`
**Library root:** `D:\Projetos\rustie\libs\rngkit-core`

## 1. Goal

Implement the approved Windows-first RngKit Tauri application and the strict
legacy-v3 CSV concatenation prerequisite in small, runnable checkpoints. Each
checkpoint must be independently reviewed, automatically validated, manually
testable where applicable, and explicitly approved by the user before the
implementer begins the next checkpoint.

Completion means:

- the reusable library supports safe derived concatenation bundles and XLSX;
- the separate app uses locked stable Tauri 2, Svelte 5, TypeScript, Vite,
  Tailwind CSS 4, and uPlot dependencies;
- Collect, Reports, Combine, and Help implement the approved behavior;
- the Rust-owned state machine and IPC boundaries are deterministically tested;
- native Windows, long-session, physical-source, and unsigned NSIS evidence is
  reported in separate tiers;
- repository-native context reflects the final verified state.

## 2. Authority and execution protocol

This plan does not itself authorize implementation. After implementation is
separately authorized, the implementer must still obey these checkpoint rules:

1. Work on only the currently approved checkpoint.
2. Preserve unrelated changes in both repositories.
3. Re-read the applicable `AGENTS.md`, context, decisions, TODO, approved
   design, and this plan at the start of each resumed implementation session.
4. Inspect current dependency documentation and local state before running a
   scaffold or changing a public contract.
5. End each checkpoint with the required automated validation, manual test
   instructions, changed-file list, unrun-test disclosure, and known later
   limitations.
6. Stop and request explicit user approval before the next checkpoint.
7. Do not batch later checkpoints because the current checkpoint completed
   quickly.
8. If implementation reveals a material contract change, stop, revise the
   design, and renew design approval before continuing.

Checkpoint approval authorizes only the implementation and local validation
described by that checkpoint. It does not authorize commit, push, remote
creation, release, code signing, publication, deployment, or remote branch
deletion. Those actions require separate explicit authorization.

## 3. Out of scope

- Multi-source, TrueRNG + BitBabbler, live XOR, reconnect, resume, or fallback.
- TrueRNGpro, driver installation, permission changes, or device management.
- RngKitPSG v2 or native-session/BIN merging.
- Bounded/downsampled frontend retention.
- P-values, significance, entropy certification, or pass/fail conclusions.
- Background tray operation.
- Additional UI component frameworks.
- Linux packaging or physical Linux support claims.
- Signed installers, releases, updater, Store submission, remote publication,
  or deployment.

## 4. Prerequisites and safeguards

- The approved design remains the product contract.
- `D:\Projetos\rustie\libs\rngkit-core` begins clean at `fe30e5b` unless the
  user presents a newer accepted state; revalidate before editing.
- `D:\Projetos\rustie\rngkit-tauri` initially contains only approved planning
  artifacts; do not overwrite them during scaffolding.
- Rust 1.85.0 is installed. Do not silently install a missing toolchain.
- Node.js, npm, Windows build tools, and Tauri prerequisites are inspected
  before scaffolding. Do not silently install missing system software.
- Default tests never enumerate or open hardware.
- Physical tests remain ignored, opt-in, and serial.
- Every final app dependency on `rngkit-*` uses the exact reachable Git
  revision produced from the validated library prerequisite. A local path may
  be used only inside the library-development checkpoints and must not remain
  in the app's final manifest or lockfile.
- The implementer must check both repositories for user changes before every
  checkpoint and work around unrelated changes rather than resetting them.

## 5. Standard checkpoint handoff

Every checkpoint handoff must contain:

```text
Checkpoint: <number and title>
Outcome: <what now works>
Changed files: <exact paths>
Automated validation: <commands and observed results>
Manual test: <numbered user steps and expected behavior>
Not run: <tests or environments not exercised, with reasons>
Known later limitations: <only items assigned to later checkpoints>
Next checkpoint: <title, not started>
Approval requested: yes
```

The implementer stops after that handoff. User feedback is fixed and the same
checkpoint is revalidated before approval is requested again.

## 6. Ordered checkpoints

### Checkpoint 1 — Derived concatenation contracts and inspection

**Repository:** `rngkit-core`

**Files to create:**

- `crates/rngkit-recording/src/concatenation/mod.rs`
- `crates/rngkit-recording/src/concatenation/naming.rs`
- `crates/rngkit-recording/src/concatenation/manifest.rs`
- `crates/rngkit-recording/src/concatenation/inspect.rs`
- `crates/rngkit-recording/tests/concatenation_inspection.rs`

**Files to modify:**

- `crates/rngkit-recording/Cargo.toml`
- `crates/rngkit-recording/src/lib.rs`
- `crates/rngkit-recording/src/error.rs`
- `crates/rngkit-recording/README.md`
- workspace `Cargo.toml` and `Cargo.lock` only for the approved SHA-256
  dependency and its locked transitive requirements
- `docs/PROJECT_CONTEXT.md`
- `docs/DECISIONS.md`
- `TODO.md`

**Actions:**

1. Add a `ConcatenationStem` parser/renderer with the approved
   `YYYYMMDDTHHMMSS_concat_<source>_s<bits>_i<seconds>[_f<fold>]` grammar and
   fold rules independent of `SessionStem`.
2. Add schema-version-1 `ConcatenationManifest` and typed input entries for
   basename, SHA-256, row count, first/last timestamp, and output range.
3. Add typed preview structures with no absolute-path serialization.
4. Add `inspect_legacy_csvs(&[PathBuf])` using a streaming reader. It validates
   legacy v3 names, nonempty inputs, one-count bounds, per-file chronological
   order, distinct canonical inputs, identical source/bits/interval/fold,
   derived chronological ordering, and nonoverlapping ranges.
5. Map failures into explicit `RecordingError` variants rather than matching
   diagnostic strings.
6. Preserve the current `open_legacy()` public behavior and default
   hardware-free tests.
7. Record the accepted derived format and compatibility rules in repository
   context.

**Acceptance criteria:**

- Preview returns stable ordered metadata and SHA-256 for valid fixtures.
- Empty, malformed, duplicate, incompatible, decreasing, and overlapping
  inputs fail with typed errors.
- Equal timestamps inside one legacy file remain accepted as nondecreasing;
  equal boundaries between two files are rejected as overlap.
- Preview/debug/serde values contain no absolute input path.
- Existing native and legacy tests remain unchanged in behavior.

**Automated validation:**

```text
cargo fmt --all -- --check
cargo test -p rngkit-recording --test concatenation_inspection
cargo test -p rngkit-recording --all-targets
cargo clippy -p rngkit-recording --all-targets -- -D warnings
cargo +1.85.0 check -p rngkit-recording --all-targets
cargo +1.85.0 test -p rngkit-recording --all-targets
git diff --check
```

**Manual user test:** Review generated fixture previews through focused test
output or an example only if an example already fits repository conventions.
No GUI exists yet. Confirm that the displayed preview contains basenames and
hashes but no absolute paths.

**Stop gate:** Do not implement bundle writing or XLSX in this checkpoint.

### Checkpoint 2 — Derived bundle creation, reading, and XLSX

**Repository:** `rngkit-core`

**Files to create:**

- `crates/rngkit-recording/src/concatenation/writer.rs`
- `crates/rngkit-recording/src/concatenation/reader.rs`
- `crates/rngkit-recording/tests/concatenation_roundtrip.rs`
- `crates/rngkit-recording/tests/concatenation_failure.rs`

**Files to modify:**

- `crates/rngkit-recording/src/concatenation/mod.rs`
- `crates/rngkit-recording/src/concatenation/manifest.rs`
- `crates/rngkit-recording/src/lib.rs`
- `crates/rngkit-recording/src/normalized.rs` only if required to construct
  the approved normalized view; do not add selectors or raw bytes
- `crates/rngkit-xlsx/src/report.rs`
- `crates/rngkit-xlsx/src/lib.rs`
- `crates/rngkit-xlsx/tests/report.rs`
- crate READMEs, `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`, and `TODO.md`

**Actions:**

1. Add `create_legacy_csv_concatenation()` and a clock-injectable test entry.
2. Reopen and completely revalidate inputs during creation rather than trusting
   preview state.
3. Stream rows into the approved normalized CSV schema, reindexing output and
   preserving input provenance.
4. Create a unique contained staging directory; sync CSV and manifest; promote
   without replacing an existing final directory; clean owned staging data on
   failure without touching inputs.
5. Add `open_concatenation()` that validates the contained same-stem CSV,
   manifest compatibility, row contiguity, input/output ranges, and one-count
   bounds, then returns a `NormalizedSession`.
6. Add `derived_report_path()` to `rngkit-xlsx`, constrained to the validated
   bundle directory, and verify `write_report()` output.
7. Add failure injection for changed-after-preview, CSV write, manifest write,
   sync, and final-promotion boundaries.
8. Verify source fixtures remain byte-for-byte unchanged.

**Acceptance criteria:**

- Valid inputs create exactly one final directory with one same-stem CSV and
  `manifest.json` and no BIN.
- The manifest contains hashes and basenames but no absolute paths.
- Derived normalized records and XLSX values match the ordered input rows.
- Existing final paths and concurrent destinations are never replaced.
- Every injected failure leaves no partial final bundle and does not mutate
  input files.
- The full six-crate deterministic workspace remains green on stable and Rust
  1.85.

**Automated validation:**

Run all commands in `rngkit-core/AGENTS.md`, including format, metadata,
workspace checks/tests, Clippy, doctests, Rust 1.85 checks/tests, feature
subsets, dependency tree, and `git diff --check`. Also run:

```text
cargo test -p rngkit-recording --test concatenation_roundtrip
cargo test -p rngkit-recording --test concatenation_failure
cargo test -p rngkit-xlsx --test report
```

**Manual user test:** Use test fixtures copied to a temporary user-selected
folder, invoke a repository-native diagnostic example only if one was added by
approved convention, inspect the CSV/manifest/XLSX, and confirm the source CSV
hashes remain unchanged. The implementer supplies exact commands based on the
implemented public API.

**Stop gate:** Stop with complete local library evidence. Do not commit, push,
or begin Tauri scaffolding.

### Authorization gate A — Reachable library revision

The final app cannot pin uncommitted local library changes. After Checkpoint 2
approval, stop and request separate authorization to commit and push the
validated `rngkit-core` changes, or request an alternative user-provided
reachable revision. Record the resulting exact revision in the next
checkpoint. Do not infer this authority from approval of Checkpoints 1 or 2.

### Checkpoint 3 — Locked Tauri/Svelte/Tailwind foundation

**Repository:** `rngkit-tauri`

**Files to create or merge from the current official Svelte-TypeScript Tauri
template:**

- `package.json`, `package-lock.json`, `tsconfig*.json`, `vite.config.ts`
- `.gitignore`, `.editorconfig`, formatting and lint configuration
- `index.html`, `src/main.ts`, `src/App.svelte`, `src/app.css`
- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/build.rs`
- `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`
- `AGENTS.md`, `README.md`, `docs/PROJECT_CONTEXT.md`,
  `docs/DECISIONS.md`, and `TODO.md`

**Existing files to preserve:**

- `docs/specs/2026-08-22-rngkit-tauri-design.md`
- `docs/plans/2026-08-22-rngkit-tauri-plan.md`

**Actions:**

1. Recheck current official Tauri, Svelte, Vite, Tailwind, and Node guidance.
2. Inspect `create-tauri-app --help`; generate the latest stable Svelte +
   TypeScript template in an isolated temporary directory, then merge only the
   reviewed template into the nonempty approved root. Do not overwrite docs.
3. Exclude prereleases. Record exact selected versions and the Node/npm floor
   in context and lockfiles.
4. Add Tailwind CSS 4 through the official Vite plugin and define initial
   CSS-first theme tokens.
5. Add uPlot and only the approved testing, linting, formatting, Tauri API, and
   native-dialog dependencies.
6. Add exact Git dependencies for the required `rngkit-*` crates at the
   reachable revision approved after gate A. No final local path dependency.
7. Set Rust edition 2024 and `rust-version = "1.85"`.
8. Keep Tauri capabilities minimal. Do not grant general filesystem, shell,
   opener, or logging access.
9. Add verified local commands and authorization boundaries to `AGENTS.md`.
10. As part of separately approved Checkpoint 3, initialize the application
    root as its own local Git repository. This authorizes only `git init`; do
    not create an initial commit or remote.

**Acceptance criteria:**

- The existing planning documents are unchanged except approved status/context
  references.
- `npm run tauri dev` opens a minimal RngKit window.
- Locked frontend and Rust builds work from the separate app repository.
- The app depends on an exact reachable library revision.
- Tailwind utilities compile and no unapproved component framework exists.
- Default app start does not enumerate hardware.

**Automated validation:**

```text
npm ci
npm run check
npm run lint
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo +1.85.0 check --manifest-path src-tauri/Cargo.toml --all-targets
```

**Manual user test:** Run `npm run tauri dev`, confirm the RngKit window opens,
theme tokens render, resizing works, and no device dialog or permission prompt
appears.

**Stop gate:** Do not build application pages or backend state yet.

### Checkpoint 4 — Responsive shell and mocked product states

**Files to create:**

- `src/components/app/AppShell.svelte`
- `src/components/app/Navigation.svelte`
- `src/components/ui/Button.svelte`
- `src/components/ui/Field.svelte`
- `src/components/ui/StatusBanner.svelte`
- `src/components/ui/MetricCard.svelte`
- `src/components/ui/Dialog.svelte`
- `src/pages/CollectPage.svelte`
- `src/pages/ReportsPage.svelte`
- `src/pages/CombinePage.svelte`
- `src/pages/HelpPage.svelte`
- `src/state/app-state.svelte.ts`
- `src/state/mock-scenarios.ts`
- `src/styles/theme.css`
- focused component and state tests under `src/**/*.test.ts`

**Files to modify:** `src/App.svelte`, `src/app.css`, frontend test config,
context, and TODO.

**Actions:**

1. Implement the four-destination shell and Tailwind token system.
2. Build semantic controls and all approved wide/narrow layouts.
3. Model idle, discovering, ready, collecting, stopping, completed, and failed
   as mocked frontend snapshots only.
4. Add a development-only scenario switch that cannot ship in production.
5. Implement theme, keyboard navigation, focus, disabled explanations, and
   descriptive-statistics copy.
6. Do not call Tauri source, collection, report, or combine commands yet.

**Acceptance criteria:** All pages and every product state are visually and
keyboard testable with mocks; no duplicate start/stop surface exists; narrow
layout stacks without page-level horizontal scrolling.

**Automated validation:** `npm run check`, `npm run lint`, frontend unit tests,
production build, and existing Rust checks.

**Manual user test:** Navigate all four pages; switch all mocked states and
themes; resize from default to minimum width; use keyboard-only navigation;
confirm the statistical warning and fold labels.

**Stop gate:** Do not add real IPC or discovery.

### Checkpoint 5 — Rust coordinator, DTOs, safe errors, and IPC seam

**Files to create:**

- `src-tauri/src/coordinator/mod.rs`
- `src-tauri/src/coordinator/state.rs`
- `src-tauri/src/dto/mod.rs`
- `src-tauri/src/dto/state.rs`
- `src-tauri/src/dto/error.rs`
- `src-tauri/src/errors.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/state.rs`
- `src-tauri/tests/state_machine.rs`
- `src/ipc/client.ts`, `src/ipc/types.ts`, and focused tests

**Files to modify:** `src-tauri/src/lib.rs`, frontend state module, context,
decisions, and TODO.

**Actions:**

1. Implement the authoritative state enum, permitted transition methods, file
   job state, session IDs, and event sequence tracking.
2. Add `get_app_state` and test-only transition commands behind development or
   test configuration; production exposes no arbitrary transition command.
3. Define tagged camel-case DTOs separately from library types.
4. Define stable safe error codes and redacted bounded diagnostic records.
5. Connect Svelte to real `get_app_state` while retaining mocked backend
   fixtures for browser tests.

**Acceptance criteria:** Rust rejects every prohibited transition; frontend
reconciles from the backend snapshot; serialized fixtures contain no selector,
entropy, seed, arbitrary error chain, or unrestricted path.

**Automated validation:** Rust unit/integration tests, frontend IPC/state tests,
format, Clippy with warnings denied, Rust 1.85 check/test, and frontend checks.

**Manual user test:** Launch the native app, verify it renders real idle state,
then use only the development scenario controls to inspect each state. Restart
and confirm production state does not inherit the development scenario.

**Stop gate:** Do not call `discover()` or create session files.

### Checkpoint 6 — Real discovery and explicit transient selection

**Files to create:**

- `src-tauri/src/discovery.rs`
- `src-tauri/src/commands/discovery.rs`
- `src-tauri/tests/discovery.rs`
- `src/components/collect/SourceDiscovery.svelte`
- `src/components/collect/SourceCandidate.svelte`
- focused frontend tests

**Files to modify:** coordinator, DTOs, command registration, Collect page,
frontend IPC/state, capabilities if narrowly required, context, decisions, and
TODO.

**Actions:**

1. Run `rngkit_sources::discover()` in Tauri's blocking context.
2. Store candidates behind random opaque tokens scoped to one generation.
3. Map candidates to safe labels, variants, ordinals, and fold requirement;
   never serialize serial or port path.
4. Map per-family issues to nonblocking safe warnings.
5. Implement selection, refresh invalidation, expired-token rejection, and
   ready-state derivation with a valid draft.
6. Inject a fake discovery service for all default tests.

**Acceptance criteria:** Partial discovery works; multiple devices stay
separate; refresh expires old tokens; no selector crosses IPC or appears in
logs/preferences; deterministic tests enumerate no hardware.

**Automated validation:** Focused discovery/state/serialization tests plus the
complete app validation baseline. Do not run ignored hardware tests by default.

**Manual user test:** Refresh sources on the Windows host, verify present
families and separate physical candidates, unplug/replug only if the user
chooses, and confirm a stale selection requires reselection. Record actual
hardware present without treating absence as failure.

**Stop gate:** Do not open a source or collect entropy.

### Checkpoint 7 — Session draft, native dialogs, and safe preferences

**Files to create:**

- `src-tauri/src/preferences.rs`
- `src-tauri/src/commands/preferences.rs`
- `src-tauri/src/commands/dialogs.rs`
- `src-tauri/tests/preferences.rs`
- `src/components/collect/SessionConfiguration.svelte`
- `src/components/app/ThemeControl.svelte`
- focused frontend tests

**Files to modify:** capabilities, coordinator, DTOs, command registration,
Collect page, frontend IPC/state, app bootstrap, context, decisions, and TODO.

**Actions:**

1. Validate sample bits, interval, fold, and output root in Rust.
2. Add the minimal native directory-dialog capability; filesystem access stays
   in Rust.
3. Implement schema-versioned atomic preferences with only approved fields.
4. Validate restored output directories and clamp geometry to visible screens.
5. Never persist candidate token or source family.

**Acceptance criteria:** Valid configuration reaches ready; invalid input does
not; safe settings survive restart; source selection never survives restart;
corrupt preferences reset with warning and do not affect session files.

**Automated validation:** Preference failure/atomicity/schema tests, DTO leak
tests, frontend form tests, full baseline.

**Manual user test:** Choose an output directory, adjust bits/interval/fold and
theme, restart, verify safe settings restore, confirm source must be freshly
selected, and move the window between monitors to test geometry recovery.

**Stop gate:** Do not start collection.

### Checkpoint 8 — Real PseudoRNG collection vertical slice

**Files to create:**

- `src-tauri/src/collection/mod.rs`
- `src-tauri/src/collection/worker.rs`
- `src-tauri/src/collection/sink.rs`
- `src-tauri/src/commands/collection.rs`
- `src-tauri/tests/collection.rs`
- `src/components/collect/CollectionControls.svelte`
- `src/components/collect/SessionSummary.svelte`
- focused frontend tests

**Files to modify:** coordinator, DTOs, commands, Collect page, IPC/state,
context, decisions, and TODO.

**Actions:**

1. Reconstruct explicit `SourceConfig` in Rust from the selected token.
2. Implement one worker, cancellation token, join-handle cleanup, session ID,
   and per-session Tauri channel.
3. Map engine events to sequenced DTOs with no raw bytes.
4. Implement Start and idempotent Stop, double-start rejection, backend
   terminal updates, and state reconciliation.
5. Exercise the real PseudoRNG adapter through the complete engine/native
   recording path; all deterministic tests use fake sources and clocks.
6. Display exact current metrics and completed/failed summary without a live
   chart yet.
7. Add a backend command that opens only the known completed session directory
   through a narrowly scoped platform opener service.

**Acceptance criteria:** A real PseudoRNG session creates one valid native
bundle; stop finalizes it; double-start fails; repeated stop is safe; channel
failure finalizes failed state; frontend receives no entropy.

**Automated validation:** Fake source/clock/channel tests for start, commit,
stop, failure, and stale events; native bundle round trip; leak checks; full
baseline.

**Manual user test:** Select PseudoRNG, collect at least three samples, stop,
open the session folder, inspect BIN/CSV/manifest presence, restart the app,
and confirm no source is preselected.

**Stop gate:** Do not add uPlot or window-close interception.

### Checkpoint 9 — Complete live chart and long-session event handling

**Files to create:**

- `src/chart/uplot-adapter.ts`
- `src/chart/chart-data.ts`
- `src/components/collect/LiveZChart.svelte`
- chart unit/component tests and synthetic dataset helpers

**Files to modify:** Collect page, frontend state, theme tokens, context, and
TODO.

**Actions:**

1. Store every accepted sample index and cumulative Z in aligned arrays.
2. Integrate uPlot once per mounted chart and clean it up on unmount.
3. Draw zero and dashed reference lines without duplicated full-length arrays.
4. Coalesce redraws to one animation frame while appending every channel
   point.
5. Implement zoom, pan, Reset view, Return to live, and zoom persistence while
   new samples arrive.
6. Add explicit descriptive labels and no inferential language.
7. Add synthetic 100,000- and 1,000,000-point stress harnesses; record rather
   than hide performance limits.

**Acceptance criteria:** Every event produces one retained point; stale or
duplicate events produce none; zoom is stable; references are correctly
labelled; no raw bytes enter frontend state; stress evidence is reproducible.

**Automated validation:** Chart data and lifecycle tests, frontend state tests,
production build, full Rust baseline. Run the synthetic stress harness on the
reference Windows host and record render, append, memory, and interaction
observations separately from deterministic pass/fail tests.

**Manual user test:** Run a PseudoRNG session, zoom while collecting, verify new
data does not reset the view, use Return to live and Reset, stop, and confirm
the point count equals committed samples.

**Stop gate:** Do not add close interception or report workflows.

### Checkpoint 10 — Graceful close, terminal recovery, and diagnostics

**Files to create:**

- `src-tauri/src/lifecycle.rs`
- `src-tauri/src/diagnostics.rs`
- `src-tauri/tests/lifecycle.rs`
- `src/components/app/CloseCollectionDialog.svelte`
- `src/components/ui/ErrorPanel.svelte`
- focused frontend tests

**Files to modify:** app bootstrap, coordinator, errors, commands, IPC/state,
Collect page, context, decisions, and TODO.

**Actions:**

1. Intercept window close in collecting/stopping states.
2. Implement Keep collecting and Stop and exit; wait for worker terminal state
   before closing.
3. Make repeated close/stop idempotent and visibly finalizing.
4. Add safe error messages, recovery actions, bounded in-memory diagnostics,
   and explicit sanitized Copy diagnostics.
5. Test frontend reload/channel loss and backend reconciliation.

**Acceptance criteria:** No approved close path silently abandons an active
session; channel loss is terminal and queryable; copied diagnostics contain no
selectors, entropy, seeds, or absolute legacy paths.

**Automated validation:** Lifecycle race tests, channel loss, close-state UI,
redaction/property tests, full baseline.

**Manual user test:** Close while collecting and exercise both choices; close
again while stopping; simulate a safe failure fixture; copy diagnostics and
inspect redaction.

**Stop gate:** Do not begin physical-source validation or reports.

### Checkpoint 11A — BitBabbler application integration validation

**Files to create or modify:** `src-tauri/tests/hardware.rs`, hardware test
helpers, context, and TODO only as required by observed app integration.

**Actions:** Add an ignored serial app-level smoke that discovers every
BitBabbler, requires explicit token selection, runs a short fold-0 session, and
validates the native bundle. Additional folds are exercised only when the user
chooses the time cost. Do not change source-crate contracts.

**Acceptance criteria:** Every present BitBabbler candidate is displayed with
its stable variant and ordinal; explicit selection runs a valid fold-0 native
session; another candidate is never silently substituted; each exercised fold
and device result is reported separately.

**Automated default validation:** Full deterministic baseline; ignored test is
compiled but not run.

**Opt-in command:**

```text
cargo test --manifest-path src-tauri/Cargo.toml --test hardware bitb -- --ignored --test-threads=1 --nocapture
```

**Manual user test:** Select each displayed BitBabbler explicitly, verify the
variant/ordinal UI and fold control, collect, stop, and inspect the manifest.

**Stop gate:** Record absent device as unverified, not passed. Stop before the
next physical family.

### Checkpoint 11B — TrueRNG application integration validation

**Files to create or modify:** Use the same hardware test, helper, context, and
TODO boundaries as Checkpoint 11A.

**Actions:** Add and run only the ignored TrueRNG smoke. Verify that multiple
ports never silently select the first candidate, then perform a short
collection and clean stop. Exercise safe disconnect behavior only if the user
explicitly agrees to the unplug test.

**Acceptance criteria:** Every present TrueRNG candidate is displayed with a
stable token; explicit selection collects a valid native bundle; a second
candidate is never silently substituted; disconnect results are classified
precisely.

**Automated default validation:** Full deterministic baseline; ignored tests
are compiled but not run.

**Opt-in command:**

```text
cargo test --manifest-path src-tauri/Cargo.toml --test hardware trng -- --ignored --test-threads=1 --nocapture
```

**Manual user test:** Select each displayed TrueRNG explicitly, collect, stop,
and inspect the manifest. Run an unplug-during-read test only after separate
agreement during this checkpoint.

**Stop gate:** Record an absent device as unverified, not passed. Stop and
request approval before RDSEED.

### Checkpoint 11C — RDSEED and unified discovery validation

**Files to create or modify:** Use the same hardware test, helper, context, and
TODO boundaries as Checkpoint 11A.

**Actions:** Add and run ignored RDSEED collection and unified discovery
smokes. Treat unsupported RDSEED as normal absence; entropy exhaustion or
instruction failure is not a skip after support was reported.

**Acceptance criteria:** Supported RDSEED collects and stops into a valid
native bundle; unsupported RDSEED is reported as absent; unified discovery
lists every actually present supported candidate without opening one.

**Automated default validation:** Full deterministic baseline; ignored tests
are compiled but not run.

**Opt-in commands:**

```text
cargo test --manifest-path src-tauri/Cargo.toml --test hardware rdseed -- --ignored --test-threads=1 --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --test hardware discover -- --ignored --test-threads=1 --nocapture
```

**Manual user test:** Confirm every actually present candidate in the UI, run
the supported RDSEED workflow if available, and inspect the completed bundle.

**Stop gate:** Report each device and OS result separately. Make no Linux
physical claim and do not begin Reports until this checkpoint is approved.

### Checkpoint 12 — Native report inspection and generation

**Files to create:**

- `src-tauri/src/reports/mod.rs`
- `src-tauri/src/reports/inspect.rs`
- `src-tauri/src/commands/reports.rs`
- `src-tauri/tests/reports_native.rs`
- `src/components/reports/ReportInput.svelte`
- `src/components/reports/ReportPreview.svelte`
- `src/components/reports/ReportActions.svelte`
- focused frontend tests

**Files to modify:** coordinator file-job state, DTOs, capabilities for native
dialogs only, Reports page, IPC/state, context, decisions, and TODO.

**Actions:**

1. Select and validate a native session directory in Rust.
2. Reject the active recording bundle and corrupt/unsupported manifests.
3. Present safe metadata and consistency warnings.
4. Generate through `native_report_path()` and `write_report()`.
5. Implement Cancel/Replace as two explicit backend requests; retain race-safe
   promotion.
6. Open only the known report or containing directory.

**Acceptance criteria:** Valid completed and interrupted committed-prefix
sessions inspect correctly; active/corrupt inputs fail safely; XLSX values and
reference labels match the library; existing output is never replaced without
explicit confirmation.

**Automated validation:** Native report command/service tests, conflict race,
file-job exclusion, frontend preview/dialog tests, full baseline.

**Manual user test:** Generate from the PseudoRNG session, inspect workbook,
generate again to exercise Cancel, then explicitly Replace and open the result.

**Stop gate:** Do not accept legacy or derived inputs yet.

### Checkpoint 13 — Legacy v3 report workflow

**Files to create:** `src-tauri/tests/reports_legacy.rs`, legacy report frontend
fixtures/tests as needed.

**Files to modify:** report inspector/commands/DTOs, Reports page, Help,
context, and TODO.

**Actions:** Detect selected legacy v3 BIN or CSV, call `open_legacy()`, show
timestamp provenance and compatibility metadata, use `legacy_report_path()`,
and preserve read-only inputs. Surface v2, partial BIN, popcount mismatch, and
one-count-bound errors safely.

**Acceptance criteria:** CSV-only, BIN-only, and consistent sibling pairs
generate correct reports without mutation; invalid fixtures fail without
partial XLSX.

**Automated validation:** Legacy fixtures for every accepted/rejected path,
input hash preservation, conflict flow, frontend tests, full baseline.

**Manual user test:** Select representative legacy CSV and BIN files from a
temporary copy, inspect provenance, generate/open XLSX, and confirm originals
remain unchanged.

**Stop gate:** Do not expose Combine yet.

### Checkpoint 14 — Combine preview, derived creation, and derived reports

**Files to create:**

- `src-tauri/src/combine/mod.rs`
- `src-tauri/src/commands/combine.rs`
- `src-tauri/tests/combine.rs`
- `src/components/combine/InputTable.svelte`
- `src/components/combine/CompatibilitySummary.svelte`
- `src/components/combine/CombineActions.svelte`
- focused frontend tests

**Files to modify:** coordinator, DTOs, Reports inspector, Combine page, Help,
IPC/state, context, decisions, and TODO.

**Actions:**

1. Select multiple legacy CSV inputs through native dialogs.
2. Preview through the library inspection API and display chronological order,
   metadata, and per-input errors.
3. Create through the library streaming API with full revalidation.
4. Display result path, row/input totals, and open-folder action.
5. Detect derived bundles in Reports, inspect their manifest, generate via
   `derived_report_path()`, and expose provenance metadata.
6. Keep absolute input paths out of persisted manifest, IPC result history,
   copied diagnostics, and production logs.

**Acceptance criteria:** Compatible inputs produce the exact derived bundle;
all specified incompatible/changed cases fail before a final bundle; derived
XLSX matches ordered rows; inputs remain unchanged.

**Automated validation:** Command/service/UI tests for success, incompatibility,
changed-after-preview, conflicts, cancellation/failure, report generation, and
path/redaction invariants; full baseline.

**Manual user test:** Combine compatible temporary legacy files, inspect the
manifest and CSV, generate XLSX, then retry with mismatched and overlapping
fixtures to verify clear rejection.

**Stop gate:** Do not start final hardening or installer work.

### Checkpoint 15 — Security, accessibility, and long-session hardening

**Files to create or modify:** capability files, all reusable UI primitives,
browser test suites, security/path tests, chart stress harness, Help, context,
decisions, and TODO.

**Actions:**

1. Audit Tauri capabilities and remove unused plugin/API access.
2. Verify every open/reveal command accepts only backend-known artifact IDs.
3. Add path containment, symlink/reparse, no-overwrite, and malformed-manifest
   adversarial fixtures within the approved library/platform boundaries.
4. Complete keyboard, focus, live-region, contrast, theme, scaling, minimum
   window, and reduced-motion coverage.
5. Run and document 100,000- and 1,000,000-point chart stress on Windows.
6. Audit user-facing strings for inferential claims and secret/selector leaks.
7. Verify production builds omit mock scenario controls and debug diagnostics.

**Acceptance criteria:** Minimal capabilities are documented; accessibility
checks and manual keyboard review pass; path and redaction adversarial tests
pass; chart retains every point and measured limitations are reported without
changing the contract.

**Automated validation:** Complete frontend and Rust suites, production build,
dependency audit tools already approved for the repository, and focused
security/accessibility checks. Do not add a new audit service silently.

**Manual user test:** Keyboard-only walkthrough of every workflow, Windows
100%/150%/200% scaling, light/dark/system themes, screen-reader status sampling,
and interactive stress-harness review.

**Stop gate:** Do not create CI workflows or build installers.

### Checkpoint 16 — Deterministic CI configuration and local parity

**Files to create:**

- `.github/workflows/ci.yml`
- any checked-in CI helper scripts justified by repeated commands

**Files to modify:** `AGENTS.md`, README, context, decisions, and TODO.

**Actions:**

1. Configure Windows and Ubuntu jobs for locked npm install, frontend checks,
   Rust stable and 1.85 checks/tests, Clippy, docs, and production frontend/Tauri
   compilation.
2. Install only documented Ubuntu Tauri build prerequisites in CI.
3. Ensure no default job runs ignored physical tests or enumerates hardware.
4. Use dependency caches without weakening lockfile resolution.
5. Reproduce the CI commands locally on the available Windows host.

**Acceptance criteria:** Workflow syntax and local parity pass; jobs separate
deterministic evidence from native/hardware/installer evidence.

**Automated validation:** Run every command encoded in CI locally. Remote CI
cannot be claimed until a separately authorized push triggers it.

**Manual user test:** Review the workflow matrix and command output; optionally
run the frontend production build and native executable locally.

**Stop gate:** Do not commit, push, trigger remote CI, or build an installer.

### Authorization gate B — Optional remote CI evidence

If the user wants remote CI evidence, request separate commit and push
authorization. After an authorized push, observe the exact run and record its
URL/result in context. A configured workflow without a run is not reported as
passing CI.

### Checkpoint 17 — Unsigned offline NSIS installer

**Files to create or modify:**

- `src-tauri/tauri.conf.json`
- reviewed application icons/assets under `src-tauri/icons/`
- installer-specific test documentation or scripts only when they eliminate
  repeated manual error
- README, `AGENTS.md`, context, decisions, and TODO

**Actions:**

1. Configure Windows 10/11 x64, English, per-user NSIS output.
2. Configure the WebView2 offline installer mode and document size impact.
3. Set product/version metadata and approved branding assets.
4. Build an unsigned NSIS package locally on Windows.
5. Test install, first launch, native dialogs, short PseudoRNG collection,
   graceful close, uninstall, and preservation of user-created session data.
6. Do not configure certificates, signing secrets, updater endpoints, release
   tokens, or publication.

**Acceptance criteria:** The unsigned installer works offline on the validated
Windows host, installs per user, launches the app, and uninstall leaves user
session output intact. SmartScreen/signing limitations are reported plainly.

**Automated validation:** Full deterministic baseline followed by:

```text
npm run tauri build -- --bundles nsis
```

Record the exact produced artifact path and hash as local evidence only.

**Manual user test:** Disconnect network if practical, install, launch, run a
short PseudoRNG session, close, uninstall, and verify the chosen output folder
still contains the session. Reconnect after the test.

**Stop gate:** Do not sign, upload, publish, release, or deploy the installer.

### Checkpoint 18 — Final context and acceptance audit

**Files to modify:** `AGENTS.md`, README, `docs/PROJECT_CONTEXT.md`,
`docs/DECISIONS.md`, `TODO.md`, and only corrections required by the acceptance
audit.

**Actions:**

1. Trace every approved design acceptance criterion to implemented code and
   evidence.
2. Run the complete deterministic suite and diff/secret/generated-artifact
   checks.
3. Summarize native, physical, remote CI, chart stress, and installer evidence
   separately, including every unverified environment.
4. Confirm no planning placeholder, stale command, secret, selector, local path
   dependency, installer artifact, or user session data is accidentally tracked.
5. Update context and TODO with exact current state and remaining separately
   authorized work.

**Acceptance criteria:** All design criteria are either evidenced or explicitly
reported as unverified; no required implementation work is silently deferred;
the repositories contain accurate handoff context.

**Automated validation:** Run the complete command sets in Section 7, the
design-to-evidence trace, tracked-file secret/generated-artifact scan, exact
Git-dependency revision check, and every separately authorized native,
physical, stress, CI, and installer check. Label checks that were not
authorized or whose environment was unavailable as unverified rather than
passing them.

**Manual user test:** Perform the complete Collect → stop → report and Combine
→ derived report workflows from a clean app launch, then review the final
evidence summary.

**Stop gate:** Implementation is complete only after this checkpoint is
approved. Do not commit, push, release, sign, publish, or deploy without a new
explicit request.

## 7. Complete validation command set

The implementer must keep exact commands current in each repository's
`AGENTS.md`. The expected final baseline is:

### Library workspace

```text
cargo fmt --all -- --check
cargo metadata --no-deps
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --doc --all-features
cargo +1.85.0 check --workspace --all-targets --all-features
cargo +1.85.0 test --workspace --all-targets --all-features
cargo tree --workspace --all-features
cargo check -p rngkit-sources --no-default-features
cargo check -p rngkit-sources --no-default-features --features bitb
cargo check -p rngkit-sources --no-default-features --features trng3
cargo check -p rngkit-sources --no-default-features --features rdseed
cargo check -p rngkit-sources --no-default-features --features pseudo
git diff --check
```

### Application repository

Exact npm script names are established in Checkpoint 3 and documented in
`AGENTS.md`. They must cover the following without changing meaning:

```text
npm ci
npm run format:check
npm run check
npm run lint
npm run test:unit -- --run
npm run test:e2e
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --doc
cargo +1.85.0 check --manifest-path src-tauri/Cargo.toml --all-targets
cargo +1.85.0 test --manifest-path src-tauri/Cargo.toml --all-targets
git diff --check
```

Browser tests use mocked IPC and no hardware. Native desktop, physical source,
remote CI, chart stress, and installer checks remain separately labelled.
Before the first separately authorized baseline commit, `git diff --check`
does not cover untracked application files; formatting, lint, and build checks
are authoritative, and the handoff must state that limitation. After a
baseline exists, `git diff --check` is mandatory.

## 8. Implementation risks and safeguards

### Cross-repository dependency

Risk: the app cannot reproducibly consume uncommitted concatenation work.
Safeguard: Gate A requires a separately authorized exact reachable revision
before app scaffolding records final dependencies.

### Long-session frontend growth

Risk: retaining every point increases memory and redraw cost indefinitely.
Safeguard: aligned numeric arrays, one chart instance, animation-frame redraw
coalescing, explicit 100,000/1,000,000-point evidence, and no silent policy
change.

### Worker and window races

Risk: double-start, stale events, channel loss, or close during blocking I/O can
produce contradictory UI state.
Safeguard: Rust-authoritative transitions, session/sequence filtering,
idempotent stop, direct worker terminal update, reconciliation command, and
focused race tests before physical integration.

### Selector and diagnostic leakage

Risk: transient hardware selectors or library diagnostics cross IPC or persist.
Safeguard: backend-only candidate registry, DTO snapshot tests, redacted bounded
diagnostics, minimal capabilities, and production-log exclusion.

### Derived artifact confusion

Risk: concatenated data is mistaken for a collected session or inputs change
after preview.
Safeguard: separate grammar/kind, CSV+manifest bundle, hashes, provenance
ranges, creation-time full revalidation, and no-overwrite staging promotion.

### Installer scope expansion

Risk: unsigned packaging drifts into signing, release, or deployment.
Safeguard: Checkpoint 17 ends at local unsigned evidence; every external or
secret-bearing action remains a separate authorization gate.

## 9. Final handoff rule

After the plan is approved, an implementation agent should receive the
approved design and this complete plan, but be told explicitly which single
checkpoint is currently authorized. The agent must not treat possession of the
whole plan as authorization to execute later checkpoints.

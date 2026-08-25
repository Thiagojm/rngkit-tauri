# RngKit Artifact Feedback and Report Chart Improvements Implementation Plan

**Status:** Approved; ready for separately authorized phased implementation

**Date:** 2026-08-25

**Approved design:**
`docs/specs/2026-08-25-rngkit-artifact-feedback-and-report-charts-design.md`

**Application root:** `D:\Projetos\rustie\rngkit-tauri`

**Library root:** `D:\Projetos\rustie\libs\rngkit-core`

## 1. Goal

Implement the approved flat-concatenation, XLSX-chart, outcome-dialog, and
working-folder contracts without weakening read-only inputs, no-overwrite
promotion, minimal Tauri capabilities, backend authority over open actions, or
descriptive-only statistical language.

Completion means:

- canonical flat legacy `_concat_` CSV files report without `manifest.json`;
- XLSX charts include the source basename and descriptive axes, use recorded
  timestamps when available, and use sample numbers for BIN-only inputs;
- collection, derived creation, and report terminal outcomes appear once in
  accessible dialogs with allowlisted complete paths and safe actions;
- Collect, Reports, and Combine have contextual no-path working-folder actions;
- Help and repository context match the implemented behavior and evidence.

## 2. Authority and execution protocol

This plan does not authorize implementation. A later generic implementation
request authorizes Phase 1 only. A request naming another phase authorizes only
that phase after its dependencies and prior gates are satisfied.

For every phase:

1. Re-read the active repository's `AGENTS.md`, context, decisions, TODO,
   approved design, and this plan.
2. Verify repository, branch, HEAD, remote relationship, and worktree before
   editing. Preserve unrelated user changes.
3. Implement only the named phase; do not scaffold later phases.
4. Run focused tests, then the applicable deterministic baseline.
5. Update project memory and public documentation only for established facts.
6. Report exact changed files, passed/unrun checks, manual test instructions,
   deviations, known limitations, and the next phase.
7. Stop for user testing and explicit authorization before continuing.

Design/plan/phase approval does not authorize later phases. Commit, push,
remote CI, library publication, dependency-pin update, installer build,
signing, release, and deployment remain separate approvals unless explicitly
included in the current request.

## 3. Baselines and safeguards

- Planning-time app baseline: published `main` at
  `a94e57509739a443a5e31feb1f46f7e8b99c22d7`, apart from these design/plan
  artifacts. Revalidate before implementation.
- Planning-time library baseline: clean published `main` at
  `2cdf311dd206cb5e7320ee520ef1e7a5139cc146`.
- Final app dependencies must use a new exact reachable Git revision, never a
  local path. Do not float unrelated dependency versions.
- Do not install toolchains, packages, or plugins silently.
- Default tests remain hardware-free. Physical tests are ignored and opt-in.
- Preserve `core:default`, `dialog:default`, and restricted CSP. Add no general
  filesystem, shell, opener, or logging capability.
- Open IPC accepts no frontend path. Full path disclosure is limited to the
  approved user-owned artifact/working-path allowlist. Device paths, selectors,
  Combine input paths, diagnostics, and logs remain redacted.
- Inputs remain read-only. Conflicts retain explicit Cancel/Replace and atomic
  no-overwrite/replace semantics.
- Z and `+/-1.96` remain descriptive, never inferential or pass/fail.

## 4. Standard phase handoff

```text
Phase: <number and title>
Outcome: <what now works>
Changed files: <exact paths>
Automated validation: <commands and observed results>
Manual user test: <steps and expected behavior>
Not run: <checks not exercised and why>
Known limitations: <items assigned to later phases>
Next phase: <title, not started>
Approval requested: yes
```

## 5. Ordered phases

### Phase 1 — Flat concatenation reader and contextual XLSX charts

**Repository:** `D:\Projetos\rustie\libs\rngkit-core`

**Goal:** Add reusable recording and workbook contracts for all later phases.

**Primary files:**

- `crates/rngkit-recording/src/concatenation/naming.rs`
- `crates/rngkit-recording/src/concatenation/inspect.rs`
- `crates/rngkit-recording/src/standalone.rs`
- `crates/rngkit-recording/src/normalized.rs`
- `crates/rngkit-recording/src/error.rs`
- `crates/rngkit-recording/src/lib.rs`
- recording README and focused tests, preferably a new
  `tests/standalone_concatenation.rs`
- `crates/rngkit-xlsx/src/report.rs`, `src/lib.rs`, `tests/report.rs`, README
- library `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`, `TODO.md`, and README

**Implementation steps:**

1. Add a distinct flat legacy concatenation classification. Do not make
   `SessionStem` accept `_concat_`; preserve its separation from
   `ConcatenationStem`.
2. Add a public open/inspect entry point that requires `.csv`, parses the
   canonical concatenation stem, rejects a current seven-column header, streams
   legacy timestamp/one-count rows, and returns `NormalizedSession`.
3. Reuse existing legacy parsing and enforce nonempty input, nondecreasing
   timestamps, supported source/fold rules, bounded one-counts, and canonical
   UTF-8 basename. Mark CSV timestamps as recorded; invent no manifest or input
   provenance.
4. Preserve `open_standalone` for ordinary session stems and expose only the
   minimum new API needed by the app resolver.
5. Add XLSX presentation options with validated source basename and X-axis mode
   (`RecordedTimestamp` or `SampleIndex`). Do not infer `.csv`/`.bin` from the
   XLSX destination.
6. Pass that context into chart construction. Use the approved filename title
   and X/Y labels. Preserve the full timestamp column for users and add a
   hidden chart-category helper column containing `HH:mm:ss` labels from the
   normalized recorded values; use the sample-index column in index mode. Do
   not invent a timezone conversion.
7. Increase chart size and style the primary line, restrained grid/background,
   zero line, and dashed `+/-1.96` references. Retain every point and avoid
   per-point markers.
8. Extend OOXML tests for titles, axes, category source, dimensions, styles,
   references, and prohibited inferential terms.
9. Cover BIN-only sample indexes and recorded-time modes for standalone CSV,
   flat concatenation, derived/native input, and BIN with valid sibling CSV.
10. Update library decisions/context/TODO only after complete validation.

**Acceptance criteria:**

- Canonical flat `_concat_` CSV normalizes and reports without a manifest.
- Stem grammars remain distinct; malformed/headered/empty/decreasing/overflow
  inputs fail without mutation or partial XLSX.
- CSV-based charts use recorded time; BIN-only charts use sample number; valid
  sibling CSV makes recorded time available.
- Chart title uses the supplied source basename and all statistical copy stays
  descriptive.

**Focused validation:**

```text
cargo test -p rngkit-recording --test standalone_concatenation
cargo test -p rngkit-recording --test standalone_inputs
cargo test -p rngkit-recording --test concatenation_inspection
cargo test -p rngkit-xlsx --test report
cargo clippy -p rngkit-recording -p rngkit-xlsx --all-targets --all-features -- -D warnings
```

Use the actual focused test target if its final filename differs. Then run the
complete library command set in Section 6.

**Manual user test:** Optional generated-workbook inspection in Excel. Native
app behavior is intentionally unavailable until later phases.

**Stop gate:** Do not commit, push, publish, update app dependencies, or start
Phase 2.

### Authorization Gate A — Make the library revision reachable

After Phase 1 approval, request explicit commit and push authorization. Record
the resulting exact reachable revision and remote CI separately. A local commit
alone does not authorize app integration; configured CI is not passing evidence
until the exact run is observed.

### Phase 2 — Application report integration

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Dependency:** Gate A is complete and the user explicitly authorizes Phase 2
and the exact dependency-pin update.

**Goal:** Inspect/report flat legacy concatenation CSVs and supply correct chart
context for every report kind.

**Primary files:**

- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
- `src-tauri/src/coordinator/state.rs`
- `src-tauri/src/reports/inspect.rs`, `src-tauri/src/reports/mod.rs`
- `src-tauri/src/commands/reports.rs`, `src-tauri/src/dto/state.rs`
- report/native/legacy/Combine/security integration tests
- report IPC types, fixtures, and focused component tests only as required

**Implementation steps:**

1. Pin every `rngkit-*` dependency to the exact Gate A revision and update the
   lockfile without floating unrelated packages.
2. Add `ReportKind::FlatLegacyConcatenation`; retain source basename and chart
   X-axis mode in backend inspected-report state.
3. Resolve parent manifest first, canonical flat concatenation second, and
   ordinary standalone input third. Corrupt manifests never fall back.
4. Preview `Legacy concatenated CSV` with recorded timestamps and a concise
   no-provenance-manifest warning.
5. Generate/revalidate a same-stem XLSX through the new library API, preserving
   conflict and explicit replacement behavior.
6. Set context for every existing report: recorded CSV timestamps use time;
   BIN-only uses indexes; selected BIN with valid timestamp CSV uses time.
7. If a sibling changes between inspection and generation, fail rather than
   silently changing axis semantics.
8. Add fixtures for success, invalid variants, immutability, conflict/replace,
   resolver precedence, and each axis decision.

**Acceptance criteria:** Flat legacy concatenation works end-to-end in Reports;
all existing inputs remain reportable; parent-manifest authority and read-only
behavior hold; no full path is added to frontend DTOs in this phase.

**Focused validation:**

```text
cargo test --locked --manifest-path src-tauri/Cargo.toml --test reports_legacy
cargo test --locked --manifest-path src-tauri/Cargo.toml --test reports_native
cargo test --locked --manifest-path src-tauri/Cargo.toml --test combine
npm run test:unit -- --run
git diff --check
```

Then run the complete app set in Section 7.

**Manual user test:** Select a copied canonical `_concat_` CSV, preview,
generate/open, exercise Cancel/Replace, and confirm input immutability. Generate
from a BIN-only copy and inspect the sample-number axis.

**Stop gate:** Do not add path DTOs, outcome notices, new folder buttons,
commit, push, or start Phase 3.

### Phase 3 — Backend outcome notices and controlled paths

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Dependency:** Phase 2 is user-validated and Phase 3 is explicitly authorized.

**Goal:** Add Rust-authoritative transient outcomes, narrowly allowlisted full
paths, acknowledgement, and no-path working-folder commands without page UI.

**Primary files:**

- `src-tauri/src/dto/state.rs`, `src-tauri/src/coordinator/state.rs`
- collection/report/Combine command and service modules
- `src-tauri/src/lib.rs` command registration
- state-machine, collection, report, Combine, and security tests

**Implementation steps:**

1. Add typed notice ID, severity, operation, path rows, and allowed action IDs
   to `AppStateDto`. Actual open authority remains backend state.
2. Retain at most one nonpersisted pending notice with a monotonic ID.
3. Create terminal collection notices only after writer reconciliation,
   listing the session directory and confirmed existing BIN/CSV/manifest files.
4. Create derived success notices with directory/CSV/manifest paths and report
   generated/replaced notices with XLSX path. Failure notices include only
   already safe relevant paths.
5. Add `acknowledge_outcome(notice_id)`; stale IDs cannot dismiss newer state.
6. Add no-path commands for Collect output root, Reports working directory, and
   context-sensitive Combine directory/output root. Validate existence before
   Explorer launch and reuse the injectable opener.
7. Replace broad “no paths in DTO” tests with an exact allowlist, while proving
   device and Combine input paths remain absent from DTOs, diagnostics, logs,
   preferences, and manifests.

**Acceptance criteria:** Approved operations create exactly one typed outcome;
choose/start actions create none; acknowledgement is race-safe; only allowlisted
paths cross IPC; every opener resolves backend-known state and accepts no path.

**Focused validation:**

```text
cargo test --locked --manifest-path src-tauri/Cargo.toml --test state_machine
cargo test --locked --manifest-path src-tauri/Cargo.toml --test collection
cargo test --locked --manifest-path src-tauri/Cargo.toml --test reports_native
cargo test --locked --manifest-path src-tauri/Cargo.toml --test reports_legacy
cargo test --locked --manifest-path src-tauri/Cargo.toml --test combine
cargo test --locked --manifest-path src-tauri/Cargo.toml --test security
git diff --check
```

Run the complete locked Rust app baseline and `npm run check`.

**Manual user test:** None; UI is intentionally deferred.

**Stop gate:** Do not render notices, add buttons, rewrite Help, commit, push,
or start Phase 4.

### Phase 4 — Outcome dialogs and working-folder UI

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Dependency:** Phase 3 is approved and Phase 4 is explicitly authorized.

**Goal:** Render outcomes once in accessible dialogs and add approved contextual
folder actions.

**Primary files:**

- `src/ipc/types.ts`, `src/ipc/client.ts`
- `src/state/app-state.svelte.ts`, `src/state/controls.ts`, `src/copy.ts`
- `src/components/ui/Dialog.svelte`
- new `src/components/app/OperationOutcomeDialog.svelte`
- `src/components/app/AppShell.svelte`
- Collect, Reports, and Combine action/configuration components
- unit/component tests and production-asset browser specs

**Implementation steps:**

1. Mirror notice types and add client functions for acknowledgement and the
   three no-path contextual folder commands.
2. Render one app-level severity-aware outcome dialog with selectable,
   monospaced, contained full paths and only backend-approved actions.
3. Acknowledge after close/action and prevent rerender, polling, hydration, or
   navigation from reopening the same ID.
4. Keep replacement and close-collection decision dialogs independent; define
   deterministic precedence so modals cannot overlap.
5. Preserve focus trap/return, Escape, keyboard actions, themes, long-path
   wrapping/scrolling, and 800x600 usability.
6. Add `Open working folder` to Collect, Reports, and Combine with backend
   disabled reasons and approved destinations. Remove only truly duplicate
   adjacent actions.
7. Present synchronous `SafeError` failures through the same visual surface
   without fabricating paths or success state.
8. Use synthetic paths in mocks and test every outcome, replacement, stale ID,
   modal precedence, navigation, folder destination, long path, keyboard, and
   theme state.

**Acceptance criteria:** Exactly one modal appears per approved terminal
outcome and none for choose/start; paths/actions are correct; acknowledgement
prevents reopening; contextual buttons resolve correctly; frontend never sends
a path to Rust.

**Focused validation:**

```text
npm run format:check
npm run check
npm run lint
npm run test:unit -- --run
npm run test:e2e
npm run build
git diff --check
```

Then run the complete app set in Section 7.

**Manual user test:** In `npm run tauri dev`, stop a short PseudoRNG collection,
create a derived bundle, generate/replace XLSX, exercise safe failures, dismiss
and navigate, and use every contextual folder button. Verify full paths,
Explorer targets, keyboard flow, and 800x600 layout.

**Stop gate:** Do not rewrite Help/context, claim unrun native evidence, commit,
push, or start Phase 5.

### Phase 5 — Help, regression audit, and native acceptance handoff

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Dependency:** Phase 4 is user-validated and Phase 5 is explicitly authorized.

**Goal:** Make guidance and project state accurate, run complete regression and
security audits, and hand off remaining native checks.

**Primary files:** `src/pages/HelpPage.svelte`, Help tests, `README.md`,
`AGENTS.md`, `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`, `TODO.md`, and
approved artifacts only for evidence-backed current-state references.

**Implementation steps:**

1. Document flat legacy concatenation, recorded-time versus BIN-only axes,
   outcome dialogs/actions, full artifact paths, and contextual folders in
   task-oriented Help.
2. Retain the concise descriptive Z boundary and remove obsolete format/path or
   inferred-time promises.
3. Trace every design criterion to code and evidence; separate deterministic,
   browser, native Excel/Explorer/Tauri, hardware, installer, and remote claims.
4. Run the complete app suite, production no-bundle Tauri build, exact library
   revision check, and tracked secret/path/generated-artifact audit.
5. Update context/decisions/TODO densely with exact observed facts and remaining
   user gates.

**Acceptance criteria:** Help is accurate and friendly; no stale contract
conflicts; all design criteria are evidenced or marked unverified; exact library
pin agrees; no unintended real path, selector, secret, XLSX, session, or
installer artifact is tracked.

**Validation:** Complete Section 7 plus:

```text
npm run tauri -- build --no-bundle -- --locked
git status --short --branch
git diff --check
```

Use read-only repository-appropriate scans for prohibited tracked content and
report exact commands. Do not add an external audit service silently.

**Manual user test:** Run the design's complete native checklist: flat legacy
CSV, BIN-only, BIN+CSV, collection completion, derived creation, report
generation/replacement, full-path dialogs, and every contextual folder.

**Stop gate:** Do not commit, push, rebuild NSIS, release, sign, publish, or
deploy without explicit authorization.

## 6. Complete library validation command set

Run from `D:\Projetos\rustie\libs\rngkit-core`:

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

## 7. Complete application validation command set

Run from `D:\Projetos\rustie\rngkit-tauri`:

```text
npm ci
npm run format:check
npm run check
npm run lint
npm run test:unit -- --run
npm run test:e2e
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --locked --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --locked --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml --doc
cargo +1.85.0 check --locked --manifest-path src-tauri/Cargo.toml --all-targets
cargo +1.85.0 test --locked --manifest-path src-tauri/Cargo.toml --all-targets
git status --short --branch
git diff --check
```

`npm run tauri -- build --no-bundle -- --locked` is required in Phase 5 and may
be run earlier for native command-registration changes. NSIS is out of scope.
Browser tests use production assets and mocked IPC, not real Tauri, Explorer,
Excel, or hardware. Ubuntu compilation is not Linux desktop support.

## 8. Risks and safeguards

### Grammar collision

Keep `SessionStem` and `ConcatenationStem` independent and use a distinct flat
reader/classification.

### Missing provenance

Label flat CSV explicitly and never imply it has manifest hashes or original
input history.

### Misleading BIN time

Fix the explicit X-axis mode during inspection and revalidate it at generation;
BIN-only uses sample numbers.

### Path disclosure expansion

Use typed allowlisted fields, backend-derived targets, no arbitrary input/device
paths, no persistence/logging/diagnostics, and exact negative security tests.

### Frontend path authority

Use action IDs and no-path commands only; displayed strings are never opener
authority.

### Repeated or competing modals

Use unique IDs, backend acknowledgement, one pending notice, deterministic
precedence, and stale-ID tests.

### Cross-repository reproducibility

Phase 1 stops before publication; Gate A separately creates a reachable exact
revision; Phase 2 updates all `rngkit-*` pins atomically.

### Excel rendering variance

Assert OOXML structure deterministically and reserve actual Excel rendering for
manual validation without claiming it from tests.

## 9. Final handoff rule

An implementation handoff cites the approved design and approved plan, names
exactly one authorized phase, and repeats its stop gate. Possession of the full
plan never authorizes later phases, commit, push, publication, release, signing,
or deployment.

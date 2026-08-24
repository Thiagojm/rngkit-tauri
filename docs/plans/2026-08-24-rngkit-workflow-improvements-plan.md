# RngKit Workflow and Compatibility Improvements Implementation Plan

**Status:** Ready for separately authorized phased implementation

**Date:** 2026-08-24

**Approved design:** `docs/specs/2026-08-24-rngkit-workflow-improvements-design.md`

**Application root:** `D:\Projetos\rustie\rngkit-tauri`

**Library root:** `D:\Projetos\rustie\libs\rngkit-core`

## 1. Goal

Implement the approved workflow and compatibility improvements without
weakening the existing collection, privacy, path-safety, provenance, or
validation contracts.

Completion means:

- `rngkit-core` recognizes real compact-timestamp legacy CSV, standalone
  current CSV, and standalone BIN inputs and creates schema-2 derived bundles
  from compatible legacy/current/mixed CSV sets;
- the app starts with `Documents/RngKit`, 2048-bit new-user defaults, and one
  automatic asynchronous discovery while preserving explicit source selection;
- Collect uses the approved instrument-style chart and one reliable `Fit all`
  action;
- Reports uses one file-oriented chooser and accepts validated bundle or
  standalone current/legacy CSV/BIN inputs;
- Combine supports incremental cross-folder CSV selection, removal, clearing,
  current/legacy/mixed validation, and schema-2 creation;
- Help is task-oriented and all repository context accurately records the new
  contracts and evidence.

## 2. Authority and execution protocol

This plan does not authorize implementation. A later implementation request
authorizes Phase 1 only unless the user explicitly names another phase.

For every phase:

1. Re-read both repositories' `AGENTS.md`, current context, decisions, TODO,
   approved design, and this plan.
2. Verify the active repository and inspect both worktrees before editing.
3. Preserve unrelated user changes and do not reset either repository.
4. Implement only the current phase. Do not add scaffolding for a later phase.
5. Run the phase's focused checks and the applicable deterministic baseline.
6. Report changed files, passed and unrun validation, user test instructions,
   known limitations, and the next phase without starting it.
7. Stop for a user validation gate. Begin the next phase only after the user
   has had an opportunity to test and gives explicit authorization.

Approval of the design, this full plan, or an earlier phase does not authorize
later phases. Commit, push, remote CI, installer rebuild, signing, release,
publication, deployment, and remote deletion remain separate approvals.

## 3. Out of scope

- BIN combination, native-bundle merging, live multi-source/XOR collection,
  fallback, reconnect, or resume.
- RngKitPSG v2 or space-delimited v2 CSV import.
- Downsampling, bounded chart retention, or statistical inference.
- General filesystem, shell, opener, or logging frontend capabilities.
- Installer signing, publishing, updater configuration, release automation, or
  deployment.
- Dependency upgrades unrelated to the approved input and UI changes.

## 4. Prerequisites and safeguards

- Application baseline is local `main` at `9103e68`, ahead of `origin/main` by
  one commit at planning time. Revalidate rather than assuming this remains
  current.
- Library baseline is clean `main` at
  `183f3c7811f5593b3b42c2558ac726552b86687d` at planning time.
- The application must never use a local path dependency for the final library
  integration. Phase 1 ends before app integration, and Authorization Gate A
  supplies a new exact reachable revision.
- Keep current locked dependency versions. Do not install or update Node, npm,
  Rust, system packages, or Tauri plugins silently.
- Default tests remain hardware-free. Physical source tests stay ignored,
  opt-in, and serial.
- The supplied representative legacy file is evidence of row shape, not a
  repository dependency. Tests create a minimal exact-format fixture and do
  not depend on `D:\OneDrive`.
- `.superpowers/` is a local brainstorming artifact. Do not commit it; add an
  ignore entry only during an authorized app phase if it is still present.

## 5. Standard phase handoff

Every phase ends with:

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

## 6. Ordered phases

### Phase 1 — Generalize standalone readers and CSV concatenation

**Repository:** `D:\Projetos\rustie\libs\rngkit-core`

**Goal:** Correct the real legacy CSV mismatch and provide the reusable,
format-neutral input and schema-2 concatenation contracts required by Reports
and Combine.

**Primary files to modify:**

- `crates/rngkit-recording/src/legacy_v3/csv.rs`
- `crates/rngkit-recording/src/legacy_v3/mod.rs`
- `crates/rngkit-recording/src/native/csv.rs`
- `crates/rngkit-recording/src/native/reader.rs`
- `crates/rngkit-recording/src/normalized.rs`
- `crates/rngkit-recording/src/concatenation/mod.rs`
- `crates/rngkit-recording/src/concatenation/inspect.rs`
- `crates/rngkit-recording/src/concatenation/manifest.rs`
- `crates/rngkit-recording/src/concatenation/writer.rs`
- `crates/rngkit-recording/src/concatenation/reader.rs`
- `crates/rngkit-recording/src/error.rs`
- `crates/rngkit-recording/src/lib.rs`
- `crates/rngkit-recording/README.md`
- `crates/rngkit-xlsx/src/report.rs` only if normalized dispatch requires it;
  do not add format parsing to the XLSX crate
- focused tests under `crates/rngkit-recording/tests/` and
  `crates/rngkit-xlsx/tests/`
- library `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`, `TODO.md`, and README

**Implementation steps:**

1. Add an exact regression fixture with stem
   `20260824T145947_bitb_s2048_i1_f0` and compact rows such as
   `20260824T145948,1014`.
2. Correct legacy timestamp parsing to accept compact v3 timestamps. Retain the
   already-supported colon-bearing form as a compatibility input and reject
   v2/ambiguous row shapes explicitly.
3. Add a format detector that reads only the required prefix and distinguishes
   exact current native CSV header from headerless legacy CSV. An almost-native
   header must fail instead of falling through to legacy parsing.
4. Add current standalone CSV normalization with contiguous index, RFC 3339
   timestamp, one-count, byte-length, and byte-offset validation.
5. Generalize standalone BIN normalization for the approved current source IDs.
   Preserve complete-sample length and same-stem sibling consistency checks.
6. Add format-neutral standalone input metadata and public normalized-reader
   entry points. Keep existing legacy entry points as safe wrappers where this
   avoids an unnecessary public break.
7. Generalize concatenation preview entries to record `legacy_v3_csv` or
   `current_csv`, using the same compatibility and chronology rules for
   legacy-only, current-only, and mixed sets.
8. Add generic inspect/create entry points. Keep legacy-only wrappers with their
   existing restrictive behavior for callers that depend on it.
9. Write new bundles as manifest schema 2, kind `csv_concatenation`, with input
   format plus existing hash/range metadata. Keep the derived CSV columns and
   directory grammar unchanged.
10. Extend `open_concatenation` to read and validate both existing schema-1
    legacy bundles and new schema-2 bundles without rewriting either.
11. Verify normalized XLSX generation for all standalone kinds and both derived
    schema versions.
12. Update library decisions/context/TODO only after tests establish the new
    contract and evidence.

**Focused acceptance criteria:**

- The representative compact legacy CSV shape opens and reports correctly.
- Current CSV-only, BIN-only, and consistent pairs normalize without a
  manifest; partial or inconsistent data fails safely.
- Legacy-only, current-only, and mixed compatible CSV sets preview and create
  schema-2 bundles with accurate input formats and hashes.
- Schema-1 derived fixtures still open and generate XLSX.
- Inputs remain byte-for-byte unchanged and serialized/debug preview/manifest
  data contains no absolute path.

**Automated validation:**

```text
cargo fmt --all -- --check
cargo metadata --no-deps
cargo test -p rngkit-recording --test legacy_v3
cargo test -p rngkit-recording --test concatenation_inspection
cargo test -p rngkit-recording --test concatenation_roundtrip
cargo test -p rngkit-recording --test concatenation_failure
cargo test -p rngkit-xlsx --test report
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

**Manual user test:** Review the focused regression output proving that the
supplied compact-timestamp shape normalizes. No GUI behavior changes in this
phase. The app remains pinned to the prior library revision until Gate A.

**Completion condition:** The complete library workspace is green on stable
and Rust 1.85, source fixtures are unchanged, and documentation reflects only
observed evidence.

**User validation gate:** Stop. Do not commit, push, change the app dependency,
or begin Phase 2.

### Authorization Gate A — Publish a reachable library revision

After Phase 1 user approval, request separate authorization to commit and push
the validated `rngkit-core` changes. Record the exact reachable commit. A local
commit without a reachable remote is insufficient for the application pin.

Do not infer commit or push authorization from Phase 1 approval. Do not begin
Phase 2 until the revision is reachable and the library worktree is verified.

### Phase 2 — Collect startup defaults and automatic discovery

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Dependency:** Authorization Gate A is complete.

**Goal:** Integrate the new exact library revision and deliver the approved
default output root, 2048-bit default, and automatic non-selecting discovery.

**Primary files to modify:**

- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`
- `src-tauri/src/lib.rs`
- `src-tauri/src/preferences.rs`
- `src-tauri/src/coordinator/state.rs`
- `src-tauri/src/coordinator/fixtures.rs`
- `src-tauri/src/dto/state.rs`
- `src-tauri/tests/preferences.rs`
- `src-tauri/tests/discovery.rs`
- `src/state/app-state.svelte.ts`
- `src/state/app-state.tauri.test.ts`
- `src/state/mock-scenarios.ts` and focused fixture tests
- `src/components/collect/SessionConfiguration.svelte` and tests
- `src/components/collect/SourceDiscovery.svelte` and tests
- `src/copy.ts` and copy audit tests
- `AGENTS.md`, README, `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`, and
  `TODO.md` only for phase-complete facts
- `.gitignore` only to exclude `.superpowers/` if still needed

**Implementation steps:**

1. Replace every `rngkit-*` dependency pin with the exact reachable Phase 1
   revision and refresh the lockfile without floating unrelated dependencies.
2. Introduce an injectable Documents-directory resolver for setup and tests.
3. When no valid saved output root exists, create and validate
   `Documents/RngKit`, keep its path backend-only, and apply it to coordinator
   and preferences state.
4. Preserve a valid saved custom root. On a missing saved root, fall back to the
   default with a short warning. On resolution/creation failure, keep startup
   usable with no root and a choose-folder recovery.
5. Change default sample bits from 8 to 2048 in preferences, coordinator,
   backend DTO fixtures, frontend mock fixtures, and expectations. Preserve any
   valid stored value.
6. After frontend hydration, invoke the existing refresh operation exactly once
   under an idle/no-candidates guard. Do not auto-select or persist a candidate.
7. Preserve manual Refresh, partial-family warnings, generation invalidation,
   prohibited-state rejection, and hardware-free default tests.
8. Update copy to describe automatic discovery and the default folder without
   exposing an absolute path.

**Focused acceptance criteria:**

- Clean/missing preferences produce 2048 bits and `RngKit` as the safe output
  label; the directory exists.
- Valid saved custom root and sample size survive restart unchanged.
- Missing custom root falls back safely; inaccessible Documents does not crash.
- Initial hydration performs one discovery, shows all fake candidates, selects
  none, and keeps Refresh operational.
- No default deterministic test calls real hardware discovery.

**Automated validation:**

```text
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
git diff --check
```

**Manual user test:** Start from a recoverable backup of current preferences,
launch the native app, confirm `Documents/RngKit` exists, 2048 is displayed,
discovery starts once, candidates remain unselected, manual Refresh works, and
a chosen custom folder persists after restart.

**Completion condition:** The native Collect configuration is usable with the
new defaults and discovery behavior; chart, Reports, Combine, and Help remain
at their prior behavior.

**User validation gate:** Stop and request approval before Phase 3.

### Phase 3 — Chart interaction fix and instrument-style redesign

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Goal:** Replace the conflicting viewport controls with `Fit all`, fix update
races, enlarge the chart, and implement the selected instrument-workspace
visual direction without changing retention.

**Primary files to modify:**

- `src/chart/uplot-adapter.ts`
- `src/chart/uplot-adapter.test.ts`
- `src/chart/chart-data.ts` only if an adapter-safe snapshot contract is needed
- `src/components/collect/LiveZChart.svelte` and tests
- `src/pages/CollectPage.svelte` and tests
- `src/styles/theme.css`
- `src/copy.ts` and `src/copy.audit.test.ts`
- chart stress helpers/tests only where required to measure the same workload
- context, decisions, and TODO after validation

**Implementation steps:**

1. Replace duplicated Svelte/adapter live state with one adapter-owned follow
   mode and a testable viewport-state callback.
2. Make pending animation-frame data updates supersedable. `Fit all` cancels or
   replaces stale pending work before framing current data.
3. Disable following on user zoom/pan. On `Fit all`, frame all retained points
   and resume following only in collecting/stopping state.
4. Remove `Reset view`, `Return to live`, their disabled reasons, and the two
   exact unwanted Collect strings.
5. Build the selected instrument monitoring card: stronger hierarchy,
   responsive 20–30 rem wide-screen chart, at least 18 rem narrow chart,
   compact integrated legend/actions, restrained grid, stronger series line,
   and readable follow/paused status.
6. Keep zero and `±1.96` reference rendering allocation-free and preserve every
   retained point.
7. Add race-focused tests for append queued before/after `Fit all`, zoom during
   collection, stop after following, repeated clicks, resize, and theme change.
8. Re-run data stress and perform a native visual/interaction pass; report
   measured evidence separately.

**Focused acceptance criteria:**

- Exactly one viewport button, `Fit all`, is present with points.
- During collection, clicking it frames all points and subsequent points remain
  visible automatically. After stop, it frames without a following mode.
- A stale queued append cannot undo the action.
- The chart is materially taller and matches the approved instrument direction
  at wide and narrow layouts.
- Removed strings are absent from production output, and retained-point count
  remains exact.

**Automated validation:** Run focused chart/component tests, `format:check`,
`check`, `lint`, the full Vitest suite, Playwright, production build, the full
locked Rust baseline, and `git diff --check`. Run the existing 100,000 and
1,000,000 data stress harness and record measurements.

**Manual user test:** Run a native PseudoRNG collection, zoom or pan to pause,
let points continue, press `Fit all`, verify current framing and continued
following, stop, press `Fit all` again, resize the window, and inspect light,
dark, and system themes.

**Completion condition:** Chart interaction and visuals pass automated and
native review without retention or statistical-contract changes.

**User validation gate:** Stop and request approval before Phase 4.

### Phase 4 — Unified Reports input and standalone file support

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Goal:** Provide one Reports chooser and route bundle artifacts or standalone
current/legacy CSV/BIN files through the generalized library reader.

**Primary files to modify:**

- `src-tauri/src/commands/dialogs.rs`
- `src-tauri/src/commands/reports.rs`
- `src-tauri/src/reports/inspect.rs`
- `src-tauri/src/reports/mod.rs`
- `src-tauri/src/coordinator/state.rs`
- `src-tauri/src/dto/state.rs`
- `src-tauri/tests/reports_legacy.rs`
- `src-tauri/tests/reports_native.rs`
- new focused standalone-input tests if clearer than expanding the two suites
- `src/ipc/client.ts`, `src/ipc/types.ts`, and tests
- `src/state/app-state.svelte.ts`
- `src/components/reports/ReportInput.svelte`
- `src/components/reports/ReportPreview.svelte`
- `src/pages/ReportsPage.svelte` and tests
- `src/copy.ts` and focused copy tests
- context, decisions, and TODO after validation

**Implementation steps:**

1. Replace the folder/file mode with one file dialog filtered to CSV, BIN, and
   JSON and labelled `Choose input`.
2. Resolve `manifest.json` and selected bundle CSV/BIN to their parent native or
   derived bundle. If a parent manifest exists, validate it authoritatively and
   do not fall back around corruption.
3. When no manifest exists, invoke the library standalone detector and map its
   normalized metadata to a safe preview.
4. Distinguish native bundle, derived bundle, current standalone CSV,
   standalone BIN, and legacy v3 CSV in user-friendly preview copy.
5. Preserve active-session rejection, same-stem report destination,
   Cancel/Replace, no-overwrite promotion, input immutability, and backend-known
   open actions.
6. Improve safe format errors without serializing detailed parser chains or
   absolute paths.
7. Add the supplied compact-format regression to app service tests using a
   generated temporary fixture.

**Focused acceptance criteria:**

- Reports shows one chooser and no folder-then-file choice.
- The supplied legacy CSV shape generates a correct XLSX.
- Standalone current CSV and BIN copies generate correct XLSX without a
  manifest; valid bundles still use full bundle validation.
- Corrupt parent bundles, malformed headers, partial BIN, and existing XLSX
  conflicts fail safely and leave inputs unchanged.

**Automated validation:** Run focused report backend/frontend tests, complete
frontend and locked Rust baselines, `git diff --check`, and explicit input hash
preservation assertions.

**Manual user test:** From the native app, select the supplied legacy CSV,
standalone current CSV, standalone current BIN, native bundle artifact, and
derived `manifest.json`; inspect each preview, generate/open XLSX, and exercise
Cancel then Replace on an existing report.

**Completion condition:** All approved Reports input shapes work through one
chooser with unchanged path/privacy protections.

**User validation gate:** Stop and request approval before Phase 5.

### Phase 5 — Mixed-format incremental Combine workflow

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Goal:** Accept legacy/current/mixed CSV sets and let users build selections
across multiple folders with Add, Remove, and Clear all.

**Primary files to modify:**

- `src-tauri/src/commands/dialogs.rs`
- `src-tauri/src/commands/combine.rs`
- `src-tauri/src/combine/mod.rs`
- `src-tauri/src/coordinator/state.rs`
- `src-tauri/src/dto/state.rs`
- `src-tauri/src/lib.rs` command registration
- `src-tauri/tests/combine.rs`
- `src-tauri/tests/security.rs`
- `src/ipc/client.ts`, `src/ipc/types.ts`, and tests
- `src/state/app-state.svelte.ts` and controls tests
- `src/components/combine/InputTable.svelte`
- `src/components/combine/CombineActions.svelte`
- `src/components/combine/CompatibilitySummary.svelte`
- `src/pages/CombinePage.svelte` and tests
- `src/copy.ts` and copy audit tests
- `AGENTS.md`, README, context, decisions, and TODO after validation

**Implementation steps:**

1. Replace the legacy-only selection state with an ordered backend map from
   opaque input IDs to canonical CSV paths and safe display ordinals.
2. Add IPC commands for dialog-backed `Add files`, remove-by-opaque-ID, and
   `Clear all`. Update the documented production command list without adding
   frontend filesystem permissions.
3. Append across repeated dialog calls and preserve prior inputs until removed
   or cleared. Remember only the backend last-dialog directory for the process;
   do not persist absolute input paths.
4. Recompute the complete generic library preview after every add/remove/clear.
   Invalidate any prior derived result when inputs change.
5. Display basename, safe ordinal, format, source, bits, interval, fold,
   timestamps, rows, and per-input/global validation state.
6. Reject canonical duplicates explicitly while allowing same-basename files
   from different folders to remain separately addressable.
7. Create schema-2 bundles through the generic library API with complete
   changed-after-preview revalidation; keep schema-1 opening/report behavior.
8. Preserve read-only inputs, no-overwrite output, backend-known open/report
   actions, and absolute-path redaction.

**Focused acceptance criteria:**

- Repeated Add operations can select inputs from different folders without
  losing prior choices.
- Remove targets exactly one opaque row; Clear all resets preview and result.
- Compatible legacy-only, current-only, and mixed sets create schema-2 bundles.
- BIN files cannot be selected or accepted by Combine.
- Duplicate, incompatible, overlapping, changed, corrupt, and same-basename
  cases display safe precise outcomes and create no partial final bundle.

**Automated validation:** Run focused Combine backend/frontend/security tests,
complete frontend and locked Rust baselines, schema-1 regression tests through
the pinned library, production build, and `git diff --check`.

**Manual user test:** Add a legacy CSV from one folder, add a compatible current
CSV from another, remove/re-add one, create and inspect the schema-2 bundle,
generate its XLSX, clear all, then test current-only, duplicate, mismatched, and
overlapping sets.

**Completion condition:** The full approved Combine workflow works natively
across folders and formats with no path leak or input mutation.

**User validation gate:** Stop and request approval before Phase 6.

### Phase 6 — Task-oriented Help, complete regression, and context audit

**Repository:** `D:\Projetos\rustie\rngkit-tauri`

**Goal:** Finish user-facing guidance, verify the integrated workflows, and
record exact current evidence without silently expanding release scope.

**Primary files to modify:**

- `src/pages/HelpPage.svelte` and tests
- `src/copy.ts` and `src/copy.audit.test.ts`
- Playwright flows under the existing e2e test location
- any focused accessibility/responsive tests required by the revised pages
- `AGENTS.md`
- README
- `docs/PROJECT_CONTEXT.md`
- `docs/DECISIONS.md`
- `TODO.md`
- approved design/plan current-state references only when evidence changes

**Implementation steps:**

1. Rewrite Help into the approved task order: Quick start, Choosing a source,
   Collecting and stopping safely, Creating reports, Combining files,
   Understanding the chart, Common problems, and File formats/version details.
2. Use short steps, direct actions, expected results, and recovery guidance.
   Keep `Z shows balance over time; it does not certify randomness.` in Help.
3. Document default folder, automatic discovery/manual selection, `Fit all`,
   standalone report inputs, mixed/incremental Combine, schema-1 compatibility,
   and schema-2 output in user-facing terms.
4. Audit all production copy for the removed strings, stale legacy-only claims,
   misleading report picker wording, inferential claims, and secret/path leaks.
5. Run the complete deterministic suite and manual integrated workflows.
6. Recheck minimal capabilities, production-only command registration, locked
   dependency pins, tracked/generated files, secrets, local path dependencies,
   and diff whitespace.
7. Update context, decisions, TODO, README, and verified commands with observed
   evidence only. Separate automated, browser, native, physical, CI, stress,
   and installer evidence.

**Focused acceptance criteria:**

- Help is understandable as a workflow guide without repository terminology.
- All revised behavior is documented consistently and obsolete copy is absent.
- Full deterministic validation is green and native user flows pass or are
  listed precisely as unverified.
- No installer, user session, input CSV/BIN, `.superpowers/`, secret, or local
  crate path is tracked accidentally.

**Automated validation:**

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
npm run tauri -- build --no-bundle -- --locked
git status --short --branch
git diff --check
```

The unsigned NSIS build, remote CI, and physical hardware tests are not part of
this phase unless separately authorized. If requested, run and report them as
separate evidence tiers.

**Manual user test:** From a normal installed or development native launch,
perform clean-start defaults/discovery, PseudoRNG Collect with chart zoom and
`Fit all`, standalone legacy/current Reports, cross-folder mixed Combine,
derived report generation, Help navigation, theme switching, keyboard-only
operation, and minimum-window responsive review.

**Completion condition:** Every approved design criterion is traced to code and
evidence, context is current, and no required work is silently deferred.

**User validation gate:** Stop. The improvement implementation is complete only
after Phase 6 user approval.

## 7. Risks and safeguards

### Format ambiguity

Risk: a malformed current header falls through to legacy parsing or a bundle
artifact bypasses its manifest. Safeguard: exact header classification,
fail-closed ambiguity, and authoritative parent-manifest validation in Reports.

### Derived compatibility drift

Risk: mixed inputs are written under misleading schema-1 provenance or old
bundles stop opening. Safeguard: schema-2/kind change for new output and
explicit dual-version read tests.

### Cross-repository reproducibility

Risk: the app integrates uncommitted local library code. Safeguard: Phase 1 and
Gate A finish before the app pin changes; the final manifest uses one exact
reachable revision and no `path=` dependency.

### Startup filesystem behavior

Risk: Documents resolution or folder creation makes launch fail or overwrites a
user preference. Safeguard: injectable resolution, valid-custom-root precedence,
safe fallback, and nonfatal recovery.

### Discovery side effects

Risk: automatic discovery becomes automatic opening/selection or enters default
tests. Safeguard: retain discovery's snapshot-only contract, explicit selection,
frontend once guard, Rust state checks, and fake discovery in deterministic
tests.

### Viewport races and long sessions

Risk: an append queued around `Fit all` restores stale viewport state or visual
polish harms million-point behavior. Safeguard: adapter-owned mode, cancellable
pending frames, race tests, no additional full arrays, and repeated stress/native
evidence.

### Cross-folder path privacy

Risk: incremental selection leaks paths or removes the wrong same-basename file.
Safeguard: backend-owned canonical paths, opaque IDs, safe ordinals, DTO leak
tests, and remove-by-ID.

## 8. Final handoff rule

An implementation handoff must name exactly one currently authorized phase.
Possession or approval of this complete plan is not authorization to implement
all phases. Phase 1 approval does not authorize Gate A; Gate A does not
authorize Phase 2; every later phase requires its preceding user validation
gate.

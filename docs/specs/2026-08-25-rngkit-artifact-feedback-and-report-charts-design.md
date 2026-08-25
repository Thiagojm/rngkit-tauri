# RngKit Artifact Feedback and Report Chart Improvements Design

**Status:** Approved

**Date:** 2026-08-25

**Application root:** `D:\Projetos\rustie\rngkit-tauri`

**Library root:** `D:\Projetos\rustie\libs\rngkit-core`

**Relationship to existing designs:** This design extends the approved
2026-08-22 baseline and 2026-08-24 workflow-improvements design. Once approved,
it supersedes only their conflicting contracts for standalone concatenation
inputs, report chart presentation, frontend path disclosure, operation outcome
dialogs, and contextual working-folder actions. All unaffected safety,
recording, statistical, and authorization boundaries remain in force.

## 1. Context and problem

The published workflow improvements support reports from collected bundles,
standalone current and legacy session files, and derived concatenation bundles.
They do not support the older flat concatenation artifact whose canonical name
contains `_concat_` but which has no `manifest.json`. The current standalone
reader sends that stem to `SessionStem`, which correctly rejects it because a
concatenation stem is intentionally not a collected-session stem.

The app also leaves successful filesystem operations represented mainly by
changed page state and enabled buttons. Users want an explicit terminal dialog
that identifies what was saved and where it was saved. The current DTO contract
intentionally withholds every absolute path, so satisfying that request requires
a narrow, explicit relaxation for user-owned working folders and artifacts.

Generated XLSX charts currently use sample indexes for every report, have a
generic title, omit axis titles, and give the primary data and descriptive
reference lines little visual hierarchy. CSV inputs contain recorded sample
timestamps and should use them. A BIN-only input does not contain timestamps;
although the library can estimate them from the filename start and configured
interval, the chart must not present those estimates as recorded sample times.

Finally, open-folder actions are currently tied to completed artifacts. Users
want a dedicated, predictable way to open the folder associated with the active
Collect, Reports, or Combine workflow.

## 2. Goals

1. Generate XLSX reports directly from canonical flat legacy concatenation CSV
   files without requiring or synthesizing a manifest.
2. Show modal outcome dialogs for terminal collection, derived-bundle, and
   report operations, including the full Windows paths of relevant artifacts.
3. Improve XLSX report charts with the source filename, descriptive axis
   titles, recorded CSV sample times when available, sample indexes when time
   is unavailable, and clearer visual hierarchy.
4. Add a dedicated contextual working-folder button to Collect, Reports, and
   Combine without accepting arbitrary frontend paths.
5. Preserve read-only inputs, race-safe output promotion, explicit replacement,
   minimal Tauri capabilities, and the descriptive-only statistical boundary.

## 3. Non-goals

- Supporting noncanonical names for flat legacy concatenation CSV files.
- Supporting a flat current-format `_concat_` CSV without a manifest.
- Combining BIN files or creating a manifest for a selected flat legacy
  concatenation CSV.
- Recovering original input-file provenance that is absent from a flat
  concatenation artifact.
- Treating filename-derived BIN timestamps as recorded measurements.
- Adding save-as dialogs, automatic Explorer launches, system notifications,
  or confirmation dialogs before starting an action.
- Showing hardware selectors, serials, device paths, entropy, seeds, arbitrary
  diagnostic chains, or Combine input paths in the frontend.
- Adding general filesystem, shell, opener, or logging capabilities.
- Changing cumulative signed Z into an inferential test, certification, or
  pass/fail result.
- Committing, pushing, publishing a library revision, releasing, signing, or
  deploying without the corresponding separate authorization.

## 4. Approved product decisions

### 4.1 Flat legacy concatenation scope

The accepted filename grammar is:

```text
YYYYMMDDTHHMMSS_concat_<source>_s<bits>_i<seconds>[_f<fold>].csv
```

The file is a headerless legacy v3 CSV containing the already concatenated
timestamp/one-count rows. It is reportable without `manifest.json`. The
filename supplies source, sample size, interval, and fold. The first and last
rows supply the actual sample range; the creation timestamp in the filename is
not substituted for row timestamps.

### 4.2 Outcome-dialog scope

Modal dialogs appear only for important terminal outcomes:

- a collection completes, is interrupted cleanly, or fails;
- a derived bundle is created or its creation fails;
- an XLSX report is generated, replaced, or generation fails.

Choosing inputs or folders and starting a collection do not add confirmation
dialogs. Existing replacement and close-while-collecting confirmations remain
separate decision dialogs. A successful outcome dialog does not automatically
open Explorer or Excel; it offers explicit actions.

### 4.3 Full-path display

Outcome dialogs display complete absolute paths. This explicitly relaxes the
previous all-paths-backend-only DTO rule for a small allowlist of user-owned
working paths and output artifacts. The relaxation does not apply to hardware
device paths, source selectors, copied diagnostics, production logs, or
arbitrary input lists.

### 4.4 Contextual working folder

- Collect opens the configured collection root, including after a session has
  completed. The existing completed-session action may still open the specific
  session directory.
- Reports opens the selected input's containing directory; after an XLSX is
  generated, this remains the report directory because output is same-folder.
- Combine opens the configured output root before creation and the created
  derived bundle directory after creation.
- Help has no working-folder button.

### 4.5 Chart source and X-axis rules

- A CSV with recorded timestamps uses those timestamps as X-axis categories,
  displayed as clock labels such as `14:59:48` from the normalized recorded
  value while retaining the full timestamp in the Samples sheet. The chart does
  not invent a timezone conversion.
- A standalone BIN with no valid same-stem timestamp-bearing CSV uses the
  one-based sample index. Its estimated timestamps remain available to existing
  normalized metadata but are not used as chart categories.
- If a selected BIN has a valid same-stem CSV, or belongs to a valid native
  bundle, the recorded CSV timestamps are available and are used.
- A flat legacy concatenation CSV uses its recorded row timestamps.
- A derived bundle uses the normalized timestamps copied from its CSV inputs.

## 5. Chosen architecture

### 5.1 Library ownership

`rngkit-recording` owns parsing and normalization of the flat legacy
concatenation CSV. It adds a format-neutral entry point for opening one
canonical `_concat_` CSV without a manifest, reusing `ConcatenationStem`, the
legacy timestamp parser, row-order validation, and one-count bounds. The result
is a `NormalizedSession`, with a distinct classification from a collected
standalone session and from a manifest-backed derived bundle.

`rngkit-xlsx` owns all workbook and chart presentation changes. Report writing
receives explicit source presentation context rather than inferring the source
extension from the XLSX destination. That context contains:

- the safe source artifact basename, including `.csv` or `.bin`;
- the X-axis mode: recorded timestamps or sample index.

The existing normalized session remains authoritative for samples, metadata,
and cumulative calculations. The app must not duplicate CSV parsing or OOXML
generation.

The library change must pass its complete stable and MSRV validation, then be
committed and made reachable at a new exact Git revision only after explicit
authorization. The app may update its dependency pin only after that revision
exists and app integration is separately authorized.

### 5.2 Report input resolution

Report resolution keeps the existing parent-manifest precedence:

1. If the selected file belongs to a directory containing `manifest.json`,
   inspect the complete native or derived bundle. A corrupt bundle never falls
   back to standalone parsing.
2. Otherwise, if the selected filename parses as `ConcatenationStem` and has a
   `.csv` extension, inspect it as a flat legacy concatenation.
3. Otherwise, use the existing standalone current/legacy CSV/BIN resolver.

The flat concatenation produces a distinct preview label such as
`Legacy concatenated CSV`, identifies that no provenance manifest is present,
and writes a same-stem `.xlsx` beside the input. It never creates, modifies, or
looks for a synthetic bundle.

The report coordinator retains the source basename and chart X-axis mode with
the backend-known inspected input so preview and generation cannot disagree.
Replacement remains a second explicit request, and generation reopens and
revalidates the input.

### 5.3 Controlled path disclosure and notices

The application DTO gains controlled presentation fields rather than exposing
coordinator filesystem state wholesale. A pending outcome notice contains:

- a monotonically unique notice ID;
- severity and operation kind;
- short title and user-friendly message;
- zero or more labeled absolute artifact paths;
- allowed actions represented by capability-like action identifiers, never by
  frontend-supplied paths.

Eligible paths are limited to the current collection root/session artifacts,
the generated report, the report directory, the derived CSV/manifest/directory,
and the contextual working directory. Every path originates from backend-owned
state after validation or successful creation. Combine source-file paths and
hardware paths remain excluded.

The frontend renders the path as selectable, wrapping monospaced text. Long
paths must not expand the dialog beyond the window. Buttons call existing or
new no-path IPC commands such as open known artifact or open known folder.

The coordinator stores at most one pending terminal notice. Dismissing it calls
an acknowledgement command with only the opaque notice ID. This prevents
polling, hydration, or rerendering from repeatedly reopening a handled dialog.
A newer terminal notice supersedes an older pending notice. Notices are
transient and are not persisted across application restarts.

Synchronous IPC failures continue to return redacted `SafeError` values; the
frontend presents them in the same outcome-dialog surface. A failure path is
included only if it was already established as a safe backend-owned working or
artifact path.

### 5.4 Collection outcomes

Clean terminal reconciliation creates one notice after the writer has finalized
the session. It identifies the final state, session stem, session directory,
and the full paths of the known CSV, BIN, and manifest entries that exist. It
offers `Open session folder` and `Close`.

An interrupted clean stop is not described as data loss: the notice reports the
committed sample count and final artifact locations. A terminal failure uses the
safe error message and recovery guidance already owned by the coordinator. It
lists only artifacts whose existence is confirmed.

### 5.5 Report and Combine outcomes

A successful report notice distinguishes `generated` from `replaced`, displays
the full XLSX path, and offers `Open report`, `Open folder`, and `Close`.

A successful Combine creation notice displays the full derived directory, CSV,
and manifest paths and offers `Open folder` and `Close`. Generating an XLSX from
the Combine page uses the standard report notice and actions.

Failure notices state that the requested artifact was not completed and must
not claim a destination exists. Existing no-overwrite and temporary-file cleanup
guarantees remain authoritative.

### 5.6 Dedicated working-folder commands

Each visible working-folder button invokes a command with no path argument:

- Collect resolves the current output root.
- Reports resolves the current inspected report directory.
- Combine resolves the derived directory when one exists, otherwise the output
  root.

Rust validates that the resolved target exists and is a directory immediately
before launching Explorer. The commands are disabled when no eligible target is
known or while an existing file job makes opening artifacts unsafe. The live
opener remains injectable for deterministic tests.

No general opener or filesystem permission is added to the frontend.

## 6. XLSX chart contract

### 6.1 Titles and labels

For a source basename `20260824T145947_bitb_s2048_i1_f0.csv`, the chart uses:

```text
Title: Z-Score Analysis — 20260824T145947_bitb_s2048_i1_f0.csv
X axis: Sample time — configured interval: 1 s
Y axis: Cumulative signed Z — sample size: 2048 bits
Primary series: Cumulative Z
```

For a BIN-only source, the X-axis title is `Sample number`; the category values
are the one-based indexes. The title still includes the selected `.bin`
basename. Axis copy is English, consistent with the rest of the application and
workbook.

### 6.2 Recorded time presentation

Timestamp-mode charts reference the Samples-sheet timestamp cells. Displayed
labels use `HH:mm:ss`; the sheet continues to retain the full normalized
timestamp so crossing midnight or combining different dates does not discard
date information. Excel may automatically reduce visible tick-label density for
large reports, but every sample remains in the series.

### 6.3 Visual hierarchy

- Increase chart dimensions and reserve sufficient worksheet space for a
  readable title, axes, and labels.
- Use a stronger blue primary line with no per-point marker by default.
- Use restrained gridlines and neutral chart/plot backgrounds compatible with
  standard Excel themes.
- Keep zero and `+/-1.96` references visually secondary; `+/-1.96` remain
  dashed.
- Keep the primary series identifiable while avoiding a legend that competes
  with the plot.
- Do not downsample or omit data rows.

The title or axes must not contain `significance`, `confidence interval`,
`pass`, `fail`, `acceptance`, `rejection`, or certification language. Existing
summary and chart safety tests remain and are extended.

## 7. Interfaces and state changes

### 7.1 `rngkit-recording`

- Add flat legacy concatenation classification and open/inspect support using
  `ConcatenationStem` without a manifest.
- Reuse the canonical legacy CSV row parser and normalized metadata types.
- Preserve timestamp provenance as recorded because timestamps come from CSV
  rows.
- Reject headers, malformed names, non-CSV extensions, empty inputs, decreasing
  timestamps, invalid folds/sources, invalid rows, and one-count overflow.

### 7.2 `rngkit-xlsx`

- Add an explicit report presentation/options type carrying source basename and
  X-axis mode.
- Pass that context through workbook construction into chart construction.
- Keep the safe atomic write and overwrite API semantics unchanged.
- Update chart OOXML and workbook-value tests for title, categories, axes,
  dimensions, formatting, reference lines, and prohibited inferential copy.

### 7.3 Tauri application

- Add a `ReportKind` variant for a flat legacy concatenation.
- Retain inspected source basename and X-axis mode in backend coordinator state.
- Add pending outcome-notice DTO/state and an acknowledgement command.
- Add allowlisted absolute artifact/working paths to the notice DTO only.
- Add or generalize no-path contextual folder-opening commands.
- Keep all file selection and every actual path resolution in Rust.

### 7.4 Frontend

- Add one reusable operation-outcome dialog capable of rendering success,
  interruption, and failure states, labeled full paths, and allowed actions.
- Add contextual `Open working folder` buttons to Collect, Reports, and Combine.
- Preserve the existing specific post-completion actions where they remain
  useful, avoiding duplicate adjacent buttons with identical destinations.
- Update Help for flat concatenation reports, dialog behavior, working folders,
  timestamp versus sample-index chart axes, and the descriptive Z boundary.

## 8. Failure and edge cases

- A `_concat_` CSV beside an unrelated or corrupt `manifest.json` is treated as
  part of that corrupt bundle and does not bypass manifest validation.
- A canonical concatenation filename with a current seven-column header is
  rejected; this feature is legacy-flat only.
- A normal collection stem is never accepted by the flat-concatenation reader.
- A malformed, empty, timestamp-decreasing, or one-count-overflow input produces
  no XLSX and does not modify the CSV.
- A flat concatenation XLSX conflict follows the existing Cancel/Replace flow.
- If a selected BIN's sibling CSV becomes unavailable or invalid before
  generation, revalidation fails rather than silently changing chart axis
  semantics.
- If a displayed artifact is removed after success, Open reports a safe error;
  the frontend path string is not treated as authority.
- Non-Unicode Windows paths must produce the existing safe unsupported/error
  behavior rather than lossy or ambiguous path disclosure.
- A dismissed notice cannot reopen from a stale snapshot. An operation finishing
  after navigation still creates one application-level notice.
- Closing a success dialog never deletes, moves, or opens an artifact.
- Explorer launch failure does not invalidate a successfully created artifact.
- Very long filenames and paths wrap or scroll within the modal while preserving
  keyboard focus and minimum-window usability.

## 9. Alternatives considered

### Implement flat concatenation parsing in Tauri

Rejected. It duplicates filename and CSV rules already owned by
`rngkit-recording` and risks divergence between Reports, Combine, and XLSX.

### Convert the flat CSV into a temporary derived bundle

Rejected. It creates unexplained filesystem artifacts, invents provenance that
the source does not contain, and complicates read-only guarantees.

### Infer the chart title and axis solely from the XLSX destination

Rejected. The destination loses whether the selected source was `.csv` or
`.bin`, and normalized BIN timestamps do not distinguish recorded time from a
user-approved sample-index presentation without explicit context.

### Show native operating-system message boxes from Rust

Rejected. Native message boxes would avoid path DTOs but provide inconsistent
layout and action handling, are harder to test, and do not integrate with the
existing accessible Svelte dialog primitive.

### Expose a general `open_path(path)` command

Rejected. It would turn frontend text into filesystem authority. Dedicated
commands resolve allowlisted backend-owned targets and accept no path.

### Keep every path backend-only and show basenames

Rejected by explicit user decision. Full artifact paths are required in outcome
dialogs, so the design narrows and tests the permitted disclosure instead.

### Use estimated BIN timestamps in the chart

Rejected by explicit user decision. Filename start plus interval is useful
normalization metadata but is not proof of actual per-sample capture time.

## 10. Validation strategy

### 10.1 Library validation

- Flat legacy concatenation success for each supported legacy source/fold rule,
  compact and compatibility timestamp rows, immutability, same-stem XLSX path,
  and normalized records.
- Rejection of noncanonical stems, current headers, BIN extension, empty files,
  malformed rows, decreasing timestamps, one-count overflow, unsupported
  sources, and invalid folds.
- Recorded-time chart fixtures for standalone CSV, flat concatenation, and
  derived/native sessions.
- Sample-index chart fixtures for BIN-only input and recorded-time fixtures when
  a valid sibling CSV is present.
- OOXML assertions for source title, X/Y axis titles, category columns, line
  hierarchy, dimensions, dashed references, and banned inferential terms.
- Full stable and Rust 1.85 library command set from its `AGENTS.md`.

### 10.2 Application backend validation

- Resolver precedence for manifest-backed bundles, flat concatenation, and
  ordinary standalone inputs.
- Preview kind, destination, source basename, axis mode, conflict, generation,
  replacement, input immutability, and revalidation races.
- Outcome creation for clean stop, interrupted stop, terminal collection
  failure, Combine success/failure, report generated/replaced/failure, notice
  replacement, and acknowledgement.
- Exact allowlist tests proving permitted artifact paths appear while device
  paths, Combine input paths, diagnostics, and logs remain redacted.
- No-path IPC signature tests for every open action and missing/deleted target
  failures.
- Full locked Rust application suite, including Clippy, docs, and MSRV.

### 10.3 Frontend and browser validation

- One modal per terminal outcome; no modal for choose/start actions.
- Full path text, wrapping/scrolling, focus entry/return, Escape/Close,
  acknowledgement, and nonreopening after rerender or hydration.
- Correct Open report/folder/session actions without path arguments.
- Contextual button destination and enabled/disabled states on all three pages.
- Production-asset browser checks at minimum and representative wide layouts,
  with light/dark/system themes and keyboard navigation.
- Full locked frontend format, type, lint, unit, E2E, and production build suite.

### 10.4 Native user validation

1. Generate a report from a canonical flat legacy `_concat_` CSV and confirm
   the input is unchanged.
2. Open the XLSX in Excel and verify the filename title, recorded-time X axis,
   descriptive Y axis, larger presentation, and all data points.
3. Generate from a BIN-only copy and verify the X axis uses sample numbers.
4. Generate from a BIN with a valid same-stem CSV and verify recorded times are
   used.
5. Complete and stop a short collection; inspect the full-path outcome dialog
   and open its session folder.
6. Create a derived bundle and generate/replace its report; verify each outcome
   dialog and action.
7. Exercise the contextual folder button before and after artifact creation in
   Collect, Reports, and Combine.

Native Excel rendering, Explorer behavior, and integrated Tauri dialogs remain
manual evidence; deterministic tests and browser mocks cannot claim them.

## 11. Delivery and authorization gates

Implementation should be divided into independently testable phases:

1. library flat-concatenation parsing and report chart context;
2. application report integration;
3. backend outcome notices and controlled paths;
4. frontend dialogs and contextual working-folder actions;
5. Help, complete regression validation, and native user-validation handoff.

Each phase stops after its documented automated validation for user review.
Approval of this design does not authorize the implementation plan, any phase,
commit, push, library publication, dependency-pin update, installer build,
release, signing, or deployment. Those boundaries must remain explicit in the
implementation plan and handoffs.

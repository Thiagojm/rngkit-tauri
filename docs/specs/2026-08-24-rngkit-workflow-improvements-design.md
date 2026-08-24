# RngKit Workflow and Compatibility Improvements Design

**Status:** Approved

**Date:** 2026-08-24

**Application root:** `D:\Projetos\rustie\rngkit-tauri`

**Library root:** `D:\Projetos\rustie\libs\rngkit-core`
**Supersedes:** The conflicting startup, standalone-input, Combine-input, and
Collect-chart portions of `docs/specs/2026-08-22-rngkit-tauri-design.md` after
this document is approved. All unaffected contracts remain in force.

## 1. Context and problem

The completed v1 baseline is usable, but native testing exposed workflow and
format mismatches that deterministic fixtures did not represent accurately.

- Collect has no usable default output root, starts with no discovery results,
  defaults to an impractically small 8-bit sample, and gives too little vertical
  space and visual hierarchy to the live chart.
- The separate `Reset view` and `Return to live` chart controls do not provide a
  useful or reliable interaction during collection.
- Reports presents separate folder and file choices even though users think in
  terms of selecting one input artifact.
- The legacy v3 CSV reader expects timestamp rows such as
  `20260824T14:59:48,1014`, while a representative real RngKitPSG v3 file uses
  `20260824T145948,1014`. Its valid stem is
  `20260824T145947_bitb_s2048_i1_f0`.
- Standalone current CSV files are rejected because their headered native schema
  is not recognized without a session bundle. Standalone current BIN files are
  not explicitly supported for all current source families.
- Combine is legacy-only by contract, rejects current CSV files, replaces the
  previous selection on every dialog use, and relies on a Windows multi-file
  dialog that cannot select files from different folders in one pass.
- Help is accurate but organized as technical reference prose instead of a
  task-oriented guide.

The solution changes reusable recording and concatenation contracts in
`rngkit-core` as well as the Tauri application. Parsing, normalization,
compatibility, provenance, and derived-bundle writing remain library-owned;
the app remains responsible for dialogs, coordinator state, IPC, and UI.

## 2. Goals

1. Make `Documents/RngKit` the safe, automatically created collection root
   when no valid saved output preference exists.
2. Start one asynchronous source discovery automatically when the app UI opens,
   while preserving explicit source selection and manual refresh.
3. Use 2048 bits as the default sample size without overwriting a valid saved
   user preference.
4. Replace the chart's two viewport controls with one reliable `Fit all`
   control and adopt the selected instrument-workspace visual direction.
5. Remove the two specified low-value Collect messages while retaining a short,
   user-friendly statistical boundary in Help.
6. Let Reports inspect one directly selected current or legacy CSV/BIN, or the
   `manifest.json`/artifact of a native or derived bundle.
7. Correct legacy v3 timestamp parsing using the observed real format.
8. Let Combine accept compatible legacy CSVs, current standalone CSVs, and
   mixed sets, but never BIN inputs.
9. Let users incrementally add Combine inputs from different folders, remove
   individual inputs, and clear the selection.
10. Rewrite Help around common user tasks and recovery actions.

## 3. Non-goals

- Combining BIN files, raw entropy bytes, or entire native bundles.
- Multi-source live collection, XOR, fallback, reconnect, or resume.
- Automatically selecting a discovered source.
- RngKitPSG v2 import or space-delimited v2 CSV support.
- Changing sample retention: every committed chart point remains retained.
- Statistical inference, randomness certification, or pass/fail conclusions.
- Exposing absolute input paths, source selectors, serials, or device paths to
  the frontend or diagnostics.
- Migrating or rewriting existing native, legacy, or derived artifacts.
- Signing, publishing, releasing, or deploying an installer.

## 4. Requirements and acceptance criteria

### 4.1 Collect defaults and startup

- On startup, the backend resolves the platform Documents directory and uses
  its `RngKit` child when no valid saved output root exists.
- The backend creates `Documents/RngKit` recursively when needed, validates it
  as a directory, and keeps the absolute path backend-only.
- A valid saved custom output root wins and remains persisted across restarts.
- If a saved custom root is unavailable, the app falls back to
  `Documents/RngKit` and shows one short safe warning.
- If Documents resolution or default-directory creation fails, startup
  continues with no output root and a safe instruction to choose a folder.
- The default sample size is 2048 bits for missing, invalid, or newly created
  preferences. A valid persisted sample size is preserved.
- After initial state hydration, the frontend triggers one asynchronous source
  refresh when the collection state is idle and has no current candidates.
- Startup discovery never opens a source, reads entropy, or selects a source.
- The existing `Refresh sources` action remains available whenever discovery is
  allowed by coordinator state.

### 4.2 Chart behavior and presentation

- `Reset view` and `Return to live` are removed and replaced by one `Fit all`
  button.
- `Fit all` immediately frames every retained point. If collection is active,
  it also resumes following future samples. If collection has ended, it only
  frames the retained data.
- User pan or zoom during collection pauses automatic following without
  stopping point ingestion. Pressing `Fit all` resumes following.
- Pending animation-frame updates cannot undo a `Fit all` request or restore a
  stale viewport mode.
- The button works during collecting, stopping, completed, and failed states
  whenever at least one point exists.
- The wide Collect layout uses an instrument-style monitoring card with a chart
  height of at least 20 rem and a responsive target near 42% of viewport
  height, capped near 30 rem. Narrow layouts retain at least 18 rem and avoid
  horizontal page scrolling.
- Metrics, collection status, an integrated compact legend, restrained grid,
  stronger cumulative-Z line, reference guides, retained-point count, and
  follow/paused state form one visual hierarchy.
- Visual polish must not allocate additional full-length data arrays or change
  the 100,000/1,000,000-point retention contract.
- The exact strings `Available after the first committed sample.` and
  `Cumulative Z and the chart lines at ±1.96 are descriptive visual references
  only. They are not a significance, pass/fail, or certification result.` are
  absent from the production Collect UI.

### 4.3 Standalone current and legacy inputs

- The library recognizes one selected file by stem and content rather than by
  requiring a manifest.
- A legacy v3 headerless CSV accepts the observed compact row timestamp
  `YYYYMMDDTHHMMSS,<ones>`. The previously accepted colon-bearing
  `YYYYMMDDTHH:MM:SS,<ones>` form remains readable for compatibility, but new
  tests and Help describe the observed compact form as canonical legacy v3.
- A current standalone CSV is detected by the exact seven-column native header
  and validated for contiguous one-based indexes, RFC 3339 timestamps, sample
  bit bounds, byte length, and contiguous byte offsets.
- A standalone BIN uses the filename stem for source, sample size, interval,
  fold, and estimated timestamps. Its length must contain only complete samples.
- Standalone current inputs support the app's current source IDs: `bitb`,
  `trng`, `rdseed`, and `pseudo`. Headerless legacy CSV remains limited to the
  source families actually represented by legacy v3.
- If a selected artifact is inside a directory with `manifest.json`, Reports
  validates the containing native or derived bundle. It does not silently
  downgrade a corrupt bundle to standalone mode. Standalone parsing is used
  when no manifest is present.
- Report inspection may use a CSV-only, BIN-only, or consistent same-stem pair.
  Existing sibling-pair consistency checks remain read-only.
- No input is repaired, renamed, moved, or modified.

### 4.4 Reports workflow

- Reports presents one `Choose input` button.
- The native file dialog accepts `.csv`, `.bin`, and `manifest.json`.
- Selecting a bundle artifact resolves to its parent bundle and validates the
  bundle. Selecting a standalone file invokes the generalized normalized
  reader.
- Preview identifies native bundle, derived bundle, current standalone CSV,
  current/legacy standalone BIN, or legacy v3 CSV, and reports timestamp
  provenance in user-friendly terms.
- Report output remains same-stem, read-only with respect to the input, and
  protected by the explicit Cancel/Replace conflict flow.
- Unsupported format errors identify whether the filename, CSV header/row
  shape, sample bounds, or bundle consistency caused rejection without exposing
  an absolute path.

### 4.5 Combine compatibility and selection

- Combine accepts CSV inputs only.
- Each input is detected as `legacy_v3_csv` or `current_csv` and normalized by
  the reusable library.
- A set may contain only legacy CSVs, only current CSVs, or both.
- All inputs must share source ID, sample bits, interval, and fold. They must be
  nonempty, individually nondecreasing, canonically distinct, and have
  nonoverlapping timestamp ranges. Equal boundary timestamps remain rejected.
- Current standalone CSV inputs do not require a manifest or BIN sibling.
  Combine validates the selected CSV itself and does not open optional siblings.
- `Add files` appends newly selected CSVs to the backend-owned selection rather
  than replacing it. Reopening the dialog can start from the most recently used
  input directory.
- Each preview row has a transient opaque input ID, safe basename, format label,
  compatibility metadata, and validation state. Absolute paths remain
  backend-only.
- `Remove` deletes one input by opaque ID. `Clear all` deletes the entire
  transient selection. Every change recomputes the full preview.
- Canonical duplicates are rejected explicitly; identical basenames from
  different folders remain distinguishable by a safe display ordinal.
- Creation reopens, rehashes, and revalidates every selected CSV before writing.

### 4.6 Derived bundle compatibility

- Existing schema-1 `legacy_csv_concatenation` bundles remain readable and
  reportable without migration.
- New creations use manifest schema 2 and kind `csv_concatenation`.
- Schema 2 adds each input's format (`legacy_v3_csv` or `current_csv`) while
  retaining basename, SHA-256, row count, first/last timestamp, and output
  range. It stores no absolute input path.
- The derived directory grammar and normalized CSV columns remain unchanged.
- The reader validates both schema versions and returns the same normalized
  session abstraction to analysis and XLSX.
- The UI calls the artifact a `Derived bundle`, not a collected session and not
  a legacy-only bundle.

### 4.7 Help and user-facing copy

- Help is organized as: Quick start, Choosing a source, Collecting and stopping
  safely, Creating reports, Combining files, Understanding the chart, Common
  problems, and File formats and version details.
- Sections use short steps, direct verbs, expected outcomes, and recovery
  actions instead of architecture terminology.
- Understanding the chart uses the concise boundary: `Z shows balance over
  time; it does not certify randomness.`
- Help documents the default folder, automatic discovery with manual source
  selection, `Fit all`, accepted standalone inputs, incremental Combine
  selection, and the difference between legacy/current/derived files.
- Stable error codes and version details remain available at the end without
  dominating the primary workflow guidance.

## 5. Chosen approach

### 5.1 Reusable library ownership

`rngkit-recording` gains a generalized standalone normalized-input detector and
generic CSV-concatenation APIs. The detector owns filename parsing, CSV format
detection, row validation, BIN reading, sibling consistency, and normalized
metadata. The generic concatenation layer owns compatibility, chronological
ordering, hashing, provenance, schema-2 writing, and schema-1/schema-2 reading.

The public surface should use format-neutral names such as
`open_standalone`, `inspect_csv_inputs`, and `create_csv_concatenation`.
Existing legacy-only functions remain as compatibility wrappers where doing so
does not weaken validation. `rngkit-xlsx` continues to consume only a
`NormalizedSession`.

This library change must be fully validated, committed, and made available at a
new exact reachable Git revision before the app replaces its current pin.
Commit and push remain separate authorization gates.

### 5.2 Application startup and preferences

The app resolves and creates the default output root in Rust during setup
through an injectable documents-directory service. Preferences schema 1 does
not need a format change: the field set is unchanged. Default construction,
missing-root recovery, and test fixtures change to 2048 bits and the resolved
default root.

Frontend hydration invokes the existing refresh workflow once under an explicit
idle/no-candidate guard. Rust authority still rejects discovery in prohibited
states, and deterministic tests continue to inject fake discovery.

### 5.3 Chart state ownership

The uPlot adapter owns one explicit following mode and its pending frame. View
commands cancel or supersede pending work atomically. Data updates reset scales
only when following is active. Pointer zoom/pan disables following. `Fit all`
sets data and scales immediately, then enables following only if the caller says
collection is active.

Svelte renders the state but does not independently maintain a conflicting
viewport mode. Presentation follows the approved instrument-workspace option
using existing Tailwind/theme tokens plus narrowly scoped uPlot CSS/plugins.

### 5.4 Reports input resolution

The report dialog returns one selected file to Rust. A resolver examines the
filename, parent manifest presence, extension, and CSV header to choose native,
derived, current standalone, or legacy standalone inspection. Bundle validation
wins whenever a parent manifest exists. Report/open commands continue to use
backend-known paths only.

### 5.5 Incremental Combine state

The coordinator stores an ordered backend-only map of opaque input IDs to
canonical paths plus the latest normalized preview. IPC adds format-neutral
commands for adding dialog-selected CSVs, removing one opaque ID, and clearing
all inputs. The frontend receives only safe rows. Adding/removing always runs a
complete preview so compatibility cannot become stale.

## 6. Interfaces and data changes

### 6.1 Library

- Add a standalone input classification enum and normalized reader that covers
  current CSV, legacy v3 CSV, and standalone BIN.
- Add generic CSV preview entries with input format and no serialized path.
- Add generic inspect/create entry points while retaining safe legacy wrappers.
- Add schema-2 manifest serialization and dual-version reading.
- Keep normalized derived CSV columns and derived stem grammar stable.

### 6.2 Tauri and frontend

- Simplify `choose_report_input` by removing the folder/file mode parameter.
- Replace the legacy-only Combine selection call with add/remove/clear commands.
- Add an opaque ID and input-format label to `CombineInputRow`.
- Keep all file paths inside Rust coordinator state.
- Update startup hydration, chart adapter contract, copy, fixtures, and control
  derivation to match the new behaviors.
- Update the documented production IPC list and minimal capability audit. No
  general filesystem capability is added.

## 7. Failure and edge cases

- Failure to resolve or create `Documents/RngKit` does not abort launch.
- Startup discovery cancellation or failure leaves Refresh available and shows
  safe partial-family warnings when applicable.
- A valid saved custom output root is never replaced merely because a default
  exists.
- A corrupt parent bundle is reported as corrupt even if its CSV could be read
  as a standalone file.
- A header that almost matches the native schema is rejected, not interpreted
  as headerless legacy data.
- Partial BIN samples, noncontiguous current indexes/offsets, invalid row
  timestamps, and one-count overflow fail before report generation or Combine
  creation.
- Adding files after a valid preview invalidates any prior derived result and
  recomputes compatibility.
- Removing one of two same-basename inputs uses its opaque ID and cannot remove
  the wrong backend path.
- Input changes between preview and creation remain a hard conflict.
- Existing schema-1 derived bundles remain read-only and do not get rewritten
  to schema 2.
- A chart data event racing with `Fit all` cannot restore the prior paused view.

## 8. Alternatives considered

### Parse current and legacy formats in Tauri

Rejected. It would duplicate recording contracts, let the desktop layer drift
from XLSX and Combine, and violate the existing reusable-library boundary.

### Convert selected files into temporary legacy files

Rejected. Conversion obscures provenance, creates unnecessary filesystem
artifacts, and risks changing timestamp or row semantics.

### Keep schema 1 and its legacy-only kind for mixed inputs

Rejected. A manifest that claims `legacy_csv_concatenation` while containing
current CSV inputs is misleading. Dual-version reading gives honest new
metadata without invalidating existing bundles.

### Require current CSVs to stay inside complete bundles

Rejected by user decision. Standalone current CSV/BIN evaluation is required;
the filename supplies metadata when a manifest is absent.

### Let one dialog scan folders recursively

Rejected. Folder scanning makes inclusion implicit, can select unrelated large
datasets, and gives weaker control than incremental explicit file selection.

### Keep separate Reset and live-follow controls

Rejected by user decision. A single `Fit all` action matches the desired mental
model and eliminates conflicting viewport modes.

## 9. Validation strategy

### 9.1 Library deterministic validation

- Add the supplied real legacy basename and compact timestamp rows as a
  read-only fixture or exact generated fixture.
- Cover compact and compatibility colon timestamps, malformed/header ambiguity,
  all supported current source IDs, current CSV-only, BIN-only, pairs, partial
  BIN, indexes, offsets, lengths, timestamp parsing, and input immutability.
- Cover legacy-only, current-only, and mixed CSV previews and creation; format
  labels; duplicates across directories; overlapping ranges; changed inputs;
  schema-2 round trip; schema-1 backward reading; hashes; provenance; staging;
  no overwrite; and no absolute paths.
- Run the complete stable and Rust 1.85 workspace commands from the library
  `AGENTS.md`. Default tests must not enumerate hardware.

### 9.2 Application backend validation

- Inject Documents resolution and filesystem failures for default-root tests.
- Verify persisted custom roots and sample sizes win, missing roots fall back,
  and new defaults are 2048 / `RngKit`.
- Test report file resolution for manifests, bundle artifacts, standalone
  current/legacy CSV, standalone BIN, corrupt bundles, and safe errors.
- Test incremental add/remove/clear, opaque IDs, same basenames from different
  directories, mixed preview, complete revalidation, result invalidation, and
  path/diagnostic redaction.
- Verify the revised production command list and unchanged minimal capability
  set.

### 9.3 Frontend and browser validation

- Verify one automatic discovery after hydration, no auto-selection, manual
  Refresh, partial warnings, and prohibited-state behavior with mocked IPC.
- Verify 2048/default-folder presentation and persisted custom values.
- Test chart interleavings between append frames, pan/zoom, `Fit all`, stop, and
  terminal state; verify one control and absence of the removed strings.
- Re-run 100,000/1,000,000-point data and native visual interaction evidence
  after chart styling changes.
- Verify one Reports chooser and incremental Combine UI across repeated mocked
  dialog calls, Remove, Clear all, duplicate basenames, keyboard use, focus, and
  responsive layout.
- Update Playwright coverage for the task-oriented Help and all revised flows.

### 9.4 Manual validation

- Launch from clean preferences; confirm `Documents/RngKit`, 2048 bits,
  automatic discovery, no automatic selection, and manual refresh.
- Collect while zoomed, press `Fit all`, and confirm framing plus continued
  following. Repeat after stop.
- Generate a report from the supplied real legacy CSV and from standalone
  current CSV/BIN copies.
- Combine legacy-only, current-only, and compatible mixed CSV sets; add files
  from at least two folders over multiple dialog openings; remove one and clear
  all; inspect the schema-2 manifest and output rows.
- Reopen an existing schema-1 derived bundle and generate its report.
- Walk through Help using keyboard-only navigation at the supported minimum
  window and normal Windows scaling.

## 10. Decisions and assumptions

- `Documents/RngKit` is the exact default path spelling.
- Valid saved output roots and sample sizes are preserved.
- Startup discovery is automatic but source selection remains explicit.
- Default sample size is 2048 bits.
- The selected chart direction is `Instrument workspace`.
- The only chart viewport action is `Fit all`.
- The two specified Collect messages are removed; Help keeps one concise
  non-certification statement.
- Reports uses one file-oriented chooser and supports standalone current or
  legacy CSV/BIN inputs.
- A parent manifest, when present, is authoritative and must validate.
- Combine accepts CSV only, including compatible current/legacy mixtures.
- Combine selection is incremental across dialogs and supports Remove/Clear.
- New derived bundles use schema 2; existing schema 1 remains supported.
- Library publication through a new reachable revision and app implementation
  are separately authorized phases. No commit, push, release, signing,
  publication, or deployment is authorized by this design.

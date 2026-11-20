# Usability and visual polish implementation plan

Status: Ready — awaiting authorization to implement. Nothing below has started.

## Goal and scope

Implement the approved design
`docs/specs/2026-11-20-rngkit-usability-polish-design.md` (UX-1 to UX-11,
section 5 wording, section 8 decisions). Completion means the 800x600 minimum
window is fully usable on Collect and Combine, Reports names its input and
output files, every status message is true for its state, and the approved
copy is applied consistently in the UI, Help, and tests.

Out of scope: new IPC commands, capabilities, dependencies, file formats,
XLSX contents, statistics, navigation redesign. Commit, push, release, and
publication remain separate approvals.

## Prerequisites

- Re-read `AGENTS.md`, `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`,
  `TODO.md`; check `git status` and preserve unrelated work.
- Use the locked toolchain and `npm ci`; add no packages.
- Baseline: run the frontend suite and `cargo test --locked --all-targets`
  before the first change and record results.
- Capture before-screenshots of all dev scenarios at 1280x800 light and
  800x600 dark (scratchpad only, not committed) for later comparison.

## Phases

Each phase is independently testable and leaves the suite green. Order is
chosen so shared building blocks (Button, theme tokens) land before the pages
that consume them, and copy changes land last to avoid churn in tests.

### Phase 1 — Button, theme tokens, and lock note (UX-5, UX-6)

Files: `src/components/ui/Button.svelte`, `Button.test.ts`,
`Button.harness.svelte`, `src/styles/theme.css`,
`src/styles/theme.contrast.test.ts`, `src/components/collect/*`.

1. Add theme tokens `--color-action-stop`, `--color-action-stop-hover`,
   `--color-action-stop-text` in all three theme blocks (light `#152033` on
   `#f4f7fb` text; dark `#e8eef5` with `#101826` text; hover one step
   lighter/darker). Register them with Tailwind's `@theme` like existing tokens.
2. Add `stop` variant to `Button`. Add a `reasonPlacement` prop:
   `'inline'` (current behavior, default for backwards compatibility) or
   `'external'` with a `describedBy` id, which renders no paragraph and sets
   `aria-describedby` to the given id. Root element no longer stretches the
   button to the reason width (`items-start`, button `self-start`).
3. Disabled secondary/ghost styling: `disabled:bg-surface-muted
   disabled:border-transparent disabled:text-text-muted
   disabled:cursor-not-allowed`. Primary/stop disabled keep current muted look.
4. New `ActionGroup.svelte` (ui): renders a flex-wrap row of buttons via a
   snippet and, below it, a deduplicated list of reasons with stable ids that
   buttons reference through `describedBy`.
5. `CollectionControls.svelte`: Stop uses `variant="stop"` plus `■` glyph
   (`aria-hidden`).
6. `SessionConfiguration.svelte` / `SourceDiscovery.svelte`: when
   `collection.state` is collecting or stopping, render one note
   (`copy.sessionLocked.collecting` / `.stopping`) at the top of Session with
   id `session-lock-note`; Refresh sources, Choose folder, and Open working
   folder use `reasonPlacement="external"` pointing to it. Other states keep
   inline reasons.

Tests: contrast ≥ 4.5:1 for stop text/background in both themes; Button
variant and external-reason `aria-describedby`; Session shows a single lock
note while collecting and no per-control lock reasons; Refresh button bounding
box identical across ready/collecting (component test via class assertion,
browser test in Phase 7).

### Phase 2 — Collect layout (UX-1, UX-7, UX-8)

Files: `src/pages/CollectPage.svelte`, `src/components/ui/MetricCard.svelte`,
`StatusBanner.svelte`, `ErrorPanel.svelte` (+ test),
`src/components/collect/SessionSummary.svelte` (+ test),
`SessionConfiguration.svelte`, `LiveZChart.svelte`.

1. Change the Collect grid breakpoint from `@[36rem]` to `@[52rem]`; verify at
   1280x800 the main container (~66rem) stays two-column and Monitoring still
   fits (2026-09-06 decision).
2. In the stacked layout, Session fields sample size and interval sit side by
   side when the Session container is ≥ 32rem (container query on Session).
3. `MetricCard`: value gets `whitespace-nowrap tabular-nums`; label uses
   `break-words` only (no mid-word breaks: `hyphens-none`, `overflow-wrap:
   normal`).
4. Re-check `LiveZChart` container-query heights after the breakpoint change so
   the stacked chart gets `min-h-[20rem]`.
5. `StatusBanner` in Monitoring: stop passing `detail`; label only.
6. `ErrorPanel`: heading becomes `copy.errorPanelHeading` ("What happened");
   message, recovery, Copy diagnostics unchanged.
7. `SessionSummary`: render the stem as a `dl` item labelled
   `copy.sessionName` ("Session name").

Tests: ErrorPanel heading; Failed scenario renders the error message exactly
once (CollectPage test); SessionSummary label; MetricCard classes.

### Phase 3 — Chart status and guide labels (UX-4, UX-10)

Files: `src/components/collect/LiveZChart.svelte` (+ test),
`src/chart/uplot-adapter.ts` (+ test), `src/copy.ts`.

1. Status phrase function of `(collectionActive, following, empty)`:
   - empty → no phrase (only `Retained points: 0` is hidden too; the
     empty-state message remains);
   - active → existing following/paused strings;
   - inactive → `copy.chart.wholeSession` ("Showing the whole session") or
     `copy.chart.zoomed` ("Zoomed view — use Fit all to see the whole
     session").
   Confirm the adapter reports `following = true` after Fit all and on initial
   data when inactive; if not, add an explicit `viewportMode` (`'full' |
   'zoomed'`) callback rather than overloading `following`.
2. `referenceLinesPlugin`: before `fillText`, measure text and fill a
   rectangle with `--color-surface` (padding 2 px × pixel ratio). Order:
   line, backdrop, text.

Tests: LiveZChart phrases for idle-empty, collecting following/paused,
completed full/zoomed, failed; paused phrase absent outside
collecting/stopping; adapter test records canvas calls and asserts
`fillRect` precedes `fillText` for each of the three labels.

### Phase 4 — Reports input and report names (UX-3, UX-6 applied)

Files: `src-tauri/src/dto/state.rs`, `src-tauri/src/reports/inspect.rs`,
`src-tauri/src/coordinator/fixtures.rs`, Rust tests under
`src-tauri/src/reports` and `src-tauri/tests`, `src/ipc/types.ts`,
`src/state/mock-scenarios.ts`, `src/components/reports/ReportPreview.svelte`,
`ReportActions.svelte` (+ tests), `src/pages/ReportsPage.test.ts`.

1. Add `input_basename: String` and `report_basename: String` to
   `ReportPreview` (serde camelCase → `inputBasename`, `reportBasename`).
   - `input_basename`: the selected artifact basename the backend already
     treats as authoritative (the value passed to
     `ReportOptions::for_session_with_source_basename`, or the native/derived
     equivalent). Never a directory path.
   - `report_basename`: `dest.file_name()` of the same-stem XLSX computed at
     inspection.
   Both use lossless UTF-8 conversion; on failure inspection returns the
   existing safe error rather than a lossy name.
2. Populate in all four inspection branches (standalone/flat, native,
   derived, legacy) and in fixtures.
3. Frontend types and mocks updated. `ReportPreview.svelte` shows **Input
   file** and **Report file** first, monospace, `break-all`, spanning both
   columns.
4. `ReportActions.svelte`: wrap in `ActionGroup` so reasons render below the
   row and buttons are sized by label.

Tests (Rust): each inspection kind asserts both basenames, that neither
contains `/` or `\`, and that `report_basename` ends with `.xlsx` and equals the
generated file's name in existing generation tests. Tests (frontend): preview
rows; ReportActions reasons rendered once below the group and referenced by
`aria-describedby`.

### Phase 5 — Combine layout and step grouping (UX-2, UX-9)

Files: `src/pages/CombinePage.svelte` (+ test),
`src/components/combine/CombineActions.svelte`, `InputTable.svelte`,
`CompatibilitySummary.svelte`.

1. `CombinePage` becomes an `@container`. Header row: title + ghost **Open
   working folder** aligned right.
2. Step 1 (above the list): Add files, Clear all.
3. `InputTable`: below `@[56rem]` render each row as a card (`li` in an `ol`
   with the same caption as `aria-label`): line 1 ordinal, basename
   (`break-all`), state, Remove; then a `dl` with format, source, bits,
   interval, fold, time range (first → last), rows. At ≥ 56rem keep the table;
   fold, timestamps, and numeric cells get `whitespace-nowrap`. Remove keeps
   its accessible name in both layouts.
4. Step 2 (below the list): incompatibility message + primary Create combined
   file.
5. Step 3: `CompatibilitySummary` result card uses
   `grid-cols-[minmax(0,2fr)_minmax(0,1fr)_minmax(0,1fr)]`, stem `break-all`,
   and contains Generate XLSX and Open folder; the card and its buttons render
   only when `combine.result` exists.
6. Replace dialog stays in `CombineActions` (or moves with the result buttons)
   without behavior change.

Tests: card vs table rendering by container width (browser), DOM order of
step buttons, Remove accessible name in card layout, result buttons absent
before a result.

### Phase 6 — Copy and Help (UX-11)

Files: `src/copy.ts`, `src/copy.audit.test.ts`, `src/pages/*.svelte`,
`src/pages/HelpPage.svelte` (+ test), all tests referencing changed strings,
`tests/e2e/*.spec.ts`.

1. Apply every row of spec section 5 through `copy` constants; move inline
   page intros (Reports, Combine) into `copy`.
2. Keep `copy.chart.title` and `copy.chart.caption` with "Descriptive"; keep
   `copy.chart.boundary` verbatim.
3. Update Help steps and control names (Create combined file, result card
   placement, Session name, Input/Report file, lock note); technical terms stay
   in "File formats and version details".
4. Extend `copy.audit.test.ts`: chart heading still contains "Descriptive";
   no "provenance-bearing" or "same-stem" in user-facing copy outside Help's
   technical section.
5. Grep tests and Help for old literals and replace with constants.

### Phase 7 — Browser coverage, docs, and validation

1. Playwright (production build, no dev scenarios available): extend
   `scaffold.spec.ts` / `accessibility.spec.ts` with 800x600 and 1280x800
   checks that are reachable without mocks (idle Collect layout, metric values
   on one line, chart width ≥ 480 px at 800x600, two columns at 1280x800, no
   overflow of `main`, Combine/Reports empty states). States that need
   snapshots stay covered by component tests.
2. Run the full verified command list from `AGENTS.md`, including MSRV and
   `npm run tauri -- build --no-bundle -- --locked`.
3. Capture after-screenshots of every dev scenario at both sizes/themes and
   compare with the baseline.
4. Update `docs/DECISIONS.md` (amend "Collect layout (2026-09-06)" breakpoint;
   Reports preview names; Combine step layout and narrow cards; neutral Stop;
   lock note; approved wording), `docs/PROJECT_CONTEXT.md` evidence, `TODO.md`,
   and set the spec and this plan to Implemented.
5. Ask the user for a native `npm run tauri dev` smoke at the minimum window
   on Windows (PseudoRNG collect/stop, Reports preview and generate, Combine
   two CSVs) and record the result only as reported.

## Risks and mitigations

- **1280x800 regression from the new breakpoint:** measure the actual main
  container width in the native window; tune between 48–56rem and keep the
  browser bounding-box test as the guard.
- **Accessibility regressions from moving reasons:** every disabled control
  keeps an `aria-describedby` target; tests assert it.
- **Chart following semantics:** if the adapter cannot distinguish full vs
  zoomed when inactive, add the explicit viewport mode callback (Phase 3).
- **DTO change:** mocks, fixtures, and `src/ipc/client.test.ts` must change in
  the same phase so typed IPC stays consistent.
- **Copy churn:** done last, through constants, to keep earlier phases'
  diffs small.

## Commit strategy (when authorized)

One commit per phase on `claude/esse-app-review-7v5qhq`, each passing the
frontend suite; Phase 4 also passes the Rust suite. Push and any pull request
only with explicit approval.

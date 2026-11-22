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
- Baseline: run the frontend suite and
  `cargo test --locked --manifest-path src-tauri/Cargo.toml --all-targets`
  before the first change and record results.
- Capture before-screenshots of all dev scenarios at 1280x800 light and
  800x600 dark (scratchpad only, not committed) for later comparison.

## Phases

Each phase is independently testable and leaves the suite green. Land shared
components and theme tokens first. Add new copy keys and update affected tests
in the phase that first uses them. Phase 6 handles the remaining wording and Help.

### Phase 1 — Button, theme tokens, and lock note (UX-5, UX-6)

Files: `src/components/ui/Button.svelte`, `Button.test.ts`,
`Button.harness.svelte`, `src/styles/theme.css`,
`src/styles/theme.contrast.test.ts`, `src/components/collect/*`, `src/copy.ts`.

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
6. In the Session configuration and source discovery components, render one
   `session-lock-note` near the top of Session, immediately after the Refresh
   sources / Device setup row, while collecting or stopping. Use the
   corresponding `copy.sessionLocked` message. Refresh sources, Choose folder,
   and Open working folder use `reasonPlacement="external"` and reference the
   note. Preserve existing reasons in other states. Render any Refresh
   sources reason below the action row to keep both buttons in place.

Test stop text contrast of at least 4.5:1 in both themes, the Button variant
and external `aria-describedby`, and one shared Session lock note referenced
by all three actions in Collecting and Stopping. Phase 7 compares
the real boxes of Refresh sources and Device setup across Ready, Collecting,
Stopping, and Completed.

### Phase 2 — Collect layout (UX-1, UX-7, UX-8)

Files: `src/pages/CollectPage.svelte`, `src/components/ui/MetricCard.svelte`,
`StatusBanner.svelte`, `ErrorPanel.svelte` (+ test),
`src/components/collect/SessionSummary.svelte` (+ test),
`SessionConfiguration.svelte`, `LiveZChart.svelte`,
`src/chart/uplot-adapter.ts` (axis sizing if needed), `src/copy.ts`.

1. Change the Collect grid breakpoint from `@[36rem]` to `@[52rem]`; verify at
   1280x800 the main container (~66rem) stays two-column and Monitoring still
   fits (2026-09-06 decision).
2. In the stacked layout, Session fields sample size and interval sit side by
   side when the Session container is ≥ 32rem (container query on Session).
3. Give `MetricCard` values `whitespace-nowrap tabular-nums`. Labels wrap
   only between words.
4. Give the stacked chart a minimum height of 20rem and enough horizontal
   space for a plot at least 480 CSS px wide at 800x600. Reduce stacked-only
   horizontal insets and tune the uPlot axis and side padding as needed,
   keeping axis labels and ticks legible. Measure the plot rectangle
   (`.u-over` in CSS pixels) with points loaded. With the current insets,
   stacking alone leaves about 498 px for the chart host before the Y axis
   and its title consume 80 px. Preserve the wide chart dimensions and
   verify that `main` has no horizontal overflow at the minimum width.
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

1. Have the adapter report `viewportMode` (`'full' | 'zoomed'`) separately
   from whether collection is active. Start in `full`, reset to `full` on
   Fit all and session reset, and report `zoomed` when a user pan or zoom
   changes the scales, including after collection ends. Keep the existing
   full-scale reset when active collection becomes terminal. Keep automatic
   following active only when collection is active and the viewport is full.
   Pending updates must respect the latest Fit all or user viewport change.
   - Empty → show the empty-state message and hide the retained point count
     and viewport phrase.
   - Active → use the existing following/paused strings.
   - Inactive with points → use `copy.chart.wholeSession` ("Showing the whole
     session") or `copy.chart.zoomed` ("Zoomed view — use Fit all to see the
     whole session"), according to the viewport mode.
2. `referenceLinesPlugin`: before `fillText`, measure text and fill a
   rectangle with `--color-surface` (padding 2 px × pixel ratio). Order:
   line, backdrop, text.

Test the LiveZChart phrases for idle-empty, collecting following/paused,
completed full/zoomed, and failed. The paused phrase is absent outside
Collecting/Stopping. Exercise zoom during collection, full view on completion,
then zoom → Fit all → zoom in the completed session. Assert the scales and
viewport mode at each step. Keep coverage for a queued update after Fit all
and clearing data for a new session. The adapter also asserts that
`fillRect` precedes `fillText` for each of the three guide labels.

### Phase 4 — Reports input and report names (UX-3, UX-6 applied)

Files: `src-tauri/src/dto/state.rs`, `src-tauri/src/reports/inspect.rs`,
`src-tauri/src/coordinator/fixtures.rs`, Rust tests under
`src-tauri/src/reports` and `src-tauri/tests`, `src/ipc/types.ts`,
`src/state/mock-scenarios.ts`, `src/components/reports/ReportPreview.svelte`,
`ReportActions.svelte` (+ tests), `src/pages/ReportsPage.test.ts`.

1. Add `input_basename: String` and `report_basename: String` to
   `ReportPreview` (serde camelCase → `inputBasename`, `reportBasename`).
   - `input_basename`: use the file selected by the user, including the
     exact `manifest.json`, `.csv`, or `.bin` choice for a bundle. Preserve its
     basename in `inspect_input` before resolving the bundle directory.
     Directory-only inspection uses the validated `manifest.json` as the
     input name.
   - `report_basename`: use `dest.file_name()` from the backend destination
     computed at inspection.
   Use lossless UTF-8 and expose only single basenames. Return an existing
   safe error if either name cannot be represented as a basename.
2. Populate the fields in standalone/flat, native, derived, and legacy
   inspection and fixtures. Keep the selected basename for preview display
   separate from the generation input and `ReportOptions`. Preserve bundle
   authority, chart timestamp selection, and workbook contents.
3. Frontend types and mocks updated. `ReportPreview.svelte` shows **Input
   file** and **Report file** first, monospace, `break-all`, spanning both
   columns.
4. `ReportActions.svelte`: wrap in `ActionGroup` so reasons render below the
   row and buttons are sized by label.

In Rust, test the chooser's inspection path with a native bundle's manifest,
CSV, and BIN, and a derived bundle's manifest and CSV. Cover standalone
current and legacy files, including a BIN with a valid CSV sibling, and a
flat `_concat_` CSV. Assert the exact selected basename, no path separators in
either field, and a predicted XLSX basename matching the generated file.
Cover directory-only inspection and keep the existing workbook and timestamp
assertions. In frontend tests, assert both preview rows and check that
ReportActions renders shared reasons once below the group and references
them with `aria-describedby`.

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

1. Keep the Playwright smoke of the unmodified production build with no mock
   installed. Add scenario coverage against the same build by injecting a
   test-only Tauri IPC mock before navigation. Reuse the typed snapshot
   fixtures and send synthetic committed-sample events through the collection
   channel for populated charts. Mock hydration, discovery, and the operations
   exercised by the tests. The production assets still omit the development
   scenario selector, and all tests run without native IPC or hardware.
2. Run the following checks at both 800x600 and 1280x800 in light and dark.
   - **Collect:** every collection state. At 800x600 verify stacking, metric
     values on one line, plot width ≥ 480 CSS px with points, and
     `main.scrollWidth <= main.clientWidth`. At 1280x800 verify two columns
     and that Monitoring fits in the default viewport in normal states.
     Compare the x, y, width, and height of Refresh sources and Device setup
     across Ready, Collecting, Stopping, and Completed.
   - **Reports:** empty state, exact input and report filenames in preview,
     and the conflict dialog's Cancel/Replace flow.
   - **Combine:** empty and populated states with compatible and incompatible
     inputs. Verify cards at 800x600 and the table on the wide layout, with
     state and Remove visible in the narrow layout. Assert the Remove
     accessible name and the result actions after creation.
3. Run the full verified command list from `AGENTS.md`, including MSRV and
   `npm run tauri -- build --no-bundle -- --locked`.
4. Capture after-screenshots of every dev scenario at both sizes/themes and
   compare with the baseline.
5. Update `docs/DECISIONS.md` (amend "Collect layout (2026-09-06)" breakpoint;
   Reports preview names; Combine step layout and narrow cards; neutral Stop;
   lock note; approved wording), `docs/PROJECT_CONTEXT.md` evidence, `TODO.md`,
   and set the spec and this plan to Implemented.
6. Ask the user for a native `npm run tauri dev` smoke at the minimum window
   on Windows (PseudoRNG collect/stop, Reports preview and generate, Combine
   two CSVs) and record the result only as reported.

## Risks and mitigations

- **1280x800 regression from the new breakpoint:** measure the actual main
  container width in the native window; tune between 48–56rem and keep the
  browser bounding-box test as the guard.
- **Accessibility regressions from moving reasons:** every disabled control
  keeps an `aria-describedby` target; tests assert it.
- **DTO change:** update mocks, fixtures, and `src/ipc/client.test.ts` in
  Phase 4 so typed IPC stays consistent.

## Commit strategy (when authorized)

One commit per phase on `claude/esse-app-review-7v5qhq`, each passing the
frontend suite; Phase 4 also passes the Rust suite. Push and any pull request
only with explicit approval.

# Usability and visual polish implementation plan

Revised 2026-11-22 after a code review against `b12fd89`.

Execution state:
- Design approved (backend wording approved 2026-11-23); user authorized
  implementing all phases with a commit and push per phase (2026-11-23).
- Phase 1 implemented; frontend suite, e2e, build, and Rust suite passed.
  `ActionGroup` deferred to Phase 4 (first user); `reasonPlacement` collapsed
  into a single `describedBy` prop.
- Phase 0 implemented: `tests/e2e/ipc-mock.ts` (init-script IPC mock,
  event emitter, viewport/box helpers) and `tests/e2e/scenarios.spec.ts`;
  e2e 8/8. Before-screenshots kept in the session scratchpad only.
- Next action: Phase 2, then Phases 3–7 in order.

## Goal and scope

Implement the approved design
`docs/specs/2026-11-20-rngkit-usability-polish-design.md` (UX-1 to UX-11,
section 5 wording, section 8 decisions). Completion means the 800x600 minimum
window is fully usable on Collect and Combine, Reports names its input and
output files, every status message is true for its state, and the approved
copy is applied consistently in the UI, Help, backend user-facing messages,
and tests.

Out of scope: new IPC commands, capabilities, dependencies, file formats,
XLSX contents, statistics, navigation redesign. Internal identifiers keep
their names, including `ReportPreview.kindLabel` values such as
`Derived bundle` (`src/state/app-state.svelte.ts` compares it). Commit, push,
release, and publication remain separate approvals.

## Prerequisites

- Re-read `AGENTS.md`, `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`,
  `TODO.md`; check `git status` and preserve unrelated work.
- Work on a branch that carries this plan, up to date with `main`.
- Use the locked toolchain and `npm ci`; add no packages.
- Baseline: run the frontend suite and
  `cargo test --locked --manifest-path src-tauri/Cargo.toml --all-targets`
  before the first change and record results.

## Phases

Each phase is independently testable and leaves the suite green. Phase 0
lands the browser harness so layout phases can measure as they go. Land
shared components and theme tokens next. Add new copy keys and update
affected tests in the phase that first uses them. Phase 6 handles the
remaining wording and Help.

### Phase 0 — Browser scenario harness

Files: `tests/e2e/` (new harness and spec files), `playwright.config.ts` only
if a project per viewport/theme is needed.

1. Keep the existing Playwright smoke of the unmodified production build with
   no mock installed.
2. Add a test-only IPC mock injected with `page.addInitScript` before
   navigation, against the same production build. `isTauri()` reads
   `globalThis.isTauri`, so the script sets it and defines
   `window.__TAURI_INTERNALS__` with `invoke`, `transformCallback`, and
   `unregisterCallback`. `Channel` delivers messages through
   `transformCallback`, so synthetic committed-sample events are sent by
   calling the registered callback. Answer `plugin:event|listen` and
   `plugin:event|unlisten` (used by `src/ipc/client.ts` via
   `@tauri-apps/api/event`).
3. `addInitScript` cannot import ESM, so write the mock as a self-contained
   script, or build it from TypeScript at test time with Vite or esbuild
   already in the lockfile. Do not add packages. Reuse the typed snapshot
   fixtures from `src/state/mock-scenarios.ts` by serialising them in the
   test process and passing them as the init-script argument.
4. Mock hydration, discovery, and only the operations the tests exercise;
   unknown commands reject loudly so missing mocks fail the test.
5. Provide helpers for viewport (800x600, 1280x800), color scheme, and element
   box measurement. Capture before-screenshots of every scenario at both sizes
   and both themes (scratchpad only, not committed).

Test: one scenario renders a collecting snapshot and a populated chart from
synthetic events; the production assets still omit the development scenario
selector; no native IPC or hardware is used.

### Phase 1 — Button, theme tokens, and lock note (UX-5, UX-6)

Files: `src/components/ui/Button.svelte`, `Button.test.ts`,
`Button.harness.svelte`, `src/styles/theme.css`,
`src/styles/theme.contrast.test.ts`, `src/components/collect/*`, `src/copy.ts`.

1. Add theme tokens in all three theme blocks (the `@theme` light block,
   `:root[data-theme='dark']`, and the system-dark media block):
   - `--color-action-stop` and `--color-action-stop-text` equal the existing
     inverse tokens (light `#152033` / `#f4f7fb`, dark `#e8eef5` /
     `#101826`). Alias with `var(--color-surface-inverse)` and
     `var(--color-text-inverse)` if Tailwind resolves it in utilities;
     otherwise use the literal values.
   - `--color-action-stop-hover`: light `#2a3a54`, dark `#c5d0de`.
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
6. In `SourceDiscovery.svelte`, render one `session-lock-note` immediately
   after the Refresh sources / Device setup row while collecting or stopping.
   Use `copy.sessionLocked.collecting` ("Settings are locked while a session
   is collecting.") or `copy.sessionLocked.stopping` ("Settings are locked
   while a session is stopping."). Refresh sources, Choose folder, and Open
   working folder use `reasonPlacement="external"` and reference the note.
   The disabled sample-size, interval, and fold inputs and the source
   fieldset also reference it through `aria-describedby`. Preserve existing
   reasons in other states. Render any Refresh sources reason (for example
   while discovering) below the action row to keep both buttons in place.

Tests:
- `theme.contrast.test.ts` asserts at least 4.5:1 for stop text on both the
  stop and the stop-hover backgrounds in both themes. It also reads
  `src/styles/theme.css` as text and checks that the three stop tokens exist
  in all three blocks with the tested values, so the literals cannot drift.
- Button variant and external `aria-describedby`.
- One shared Session lock note referenced by all three actions and the
  disabled inputs in Collecting and Stopping.
- Browser (Phase 0 harness): the boxes of Refresh sources and Device setup are
  identical across Ready, Collecting, Stopping, and Completed.

### Phase 2 — Collect layout (UX-1, UX-7, UX-8)

Files: `src/pages/CollectPage.svelte`, `src/components/ui/MetricCard.svelte`,
`StatusBanner.svelte`, `ErrorPanel.svelte` (+ test),
`src/components/collect/SessionSummary.svelte` (+ test),
`SessionConfiguration.svelte`, `LiveZChart.svelte`,
`src/chart/uplot-adapter.ts` (axis sizing if needed), `src/copy.ts`.

Root cause to fix: the metric grid (`@min-[28rem]:grid-cols-3`) and the chart
sizing in `LiveZChart.svelte` (`@[36rem]:…`) resolve against the outer Collect
`@container`, not the Monitoring column. At 800x600 they therefore select the
wide variants inside a ~220 px column.

1. Make the Monitoring `section` its own named container
   (`@container/monitor`) and key the metric grid and the chart sizing
   classes on it (`@min-[28rem]/monitor:`, `@[36rem]/monitor:`). Re-tune
   those thresholds against the Monitoring width, not the page width.
2. Change the Collect grid breakpoint from `@[36rem]` to `@[52rem]`; verify at
   1280x800 the main container (~66rem) stays two-column and Monitoring still
   fits (2026-09-06 decision). Measure with the Phase 0 harness while tuning.
3. In the stacked layout, Session fields sample size and interval sit side by
   side when the Session container is ≥ 32rem (named container on Session).
4. Give `MetricCard` values `whitespace-nowrap tabular-nums` (replacing
   `break-words`). Labels wrap only between words.
5. Give the stacked chart a minimum height of 20rem and enough horizontal
   space for a plot at least 480 CSS px wide at 800x600. Reduce stacked-only
   horizontal insets and tune the uPlot axis and side padding as needed,
   keeping axis labels and ticks legible. Measure the plot rectangle
   (`.u-over` in CSS pixels) with points loaded. With the current insets,
   stacking alone leaves about 498 px for the chart host before the Y axis
   and its title consume 80 px. Preserve the wide chart dimensions and
   verify that `main` has no horizontal overflow at the minimum width.
6. `StatusBanner` in Monitoring: stop passing `detail`; label only.
7. `ErrorPanel`: heading becomes `copy.errorPanelHeading` ("What happened");
   message, recovery, Copy diagnostics unchanged.
8. `SessionSummary`: render the stem as a `dl` item labelled
   `copy.sessionName` ("Session name").

Tests: ErrorPanel heading; Failed scenario renders the error message exactly
once (CollectPage test); SessionSummary label; MetricCard classes. Browser:
at 800x600 stacked layout, one-line metric values, plot ≥ 480 px; at
1280x800 two columns unchanged.

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
   - uPlot's built-in cursor (`drag.setScale: true`) also changes scales on
     its own: drag-to-zoom and the default double-click reset to autoscale,
     which today does not notify the adapter. Disable the default
     `cursor.bind.dblclick` and route double-click on `plot.over` to the
     adapter's Fit all, so it sets mode `full` and resumes following while
     collection is active.
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
then zoom → Fit all → zoom → double-click in the completed session, and zoom →
double-click while collecting (following resumes). Assert the scales and
viewport mode at each step. Keep coverage for a queued update after Fit all
and clearing data for a new session. The adapter also asserts that
`fillRect` precedes `fillText` for each of the three guide labels.

### Phase 4 — Reports input and report names (UX-3, UX-6 applied)

Files: `src-tauri/src/dto/state.rs`, `src-tauri/src/reports/inspect.rs`,
`src-tauri/src/combine/mod.rs`, `src-tauri/src/coordinator/fixtures.rs`, Rust
tests under `src-tauri/src/reports` and `src-tauri/tests`, `src/ipc/types.ts`,
`src/state/mock-scenarios.ts`, `src/components/reports/ReportPreview.svelte`,
`ReportActions.svelte` (+ tests), `src/pages/ReportsPage.test.ts`.

1. Add `input_basename: String` and `report_basename: String` to
   `ReportPreview` (serde camelCase → `inputBasename`, `reportBasename`).
   - Every inspection function fills both fields itself: `inspect_native`
     and `inspect_derived` (directory inputs) use `manifest.json`;
     `inspect_standalone` and `inspect_legacy` use the input file name; all
     use `dest.file_name()` for `report_basename`. Public signatures stay
     unchanged, so callers in `src-tauri/tests` and `combine/mod.rs` keep
     compiling.
   - `inspect_input` captures `path.file_name()` before resolving the bundle
     directory and, for bundle selections, overwrites `input_basename` with
     the exact `manifest.json`, `.csv`, or `.bin` the user chose. The chooser
     (`pick_file`) only returns files.
   - Combine paths (`generate_derived_report` and `finish_created` in
     `src-tauri/src/combine/mod.rs`) call `inspect_derived` on the bundle
     directory and set the Reports preview through `set_inspected_report`;
     they show `manifest.json` as the input name.
   Use lossless UTF-8 and expose only single basenames. Return an existing
   safe error if either name cannot be represented as a basename.
2. Update fixtures. Keep the selected basename for preview display separate
   from the generation input and `ReportOptions`. Preserve bundle authority,
   chart timestamp selection, and workbook contents.
3. Frontend types and mocks updated. `ReportPreview.svelte` shows **Input
   file** and **Report file** first, monospace, `break-all`, spanning both
   columns.
4. `ReportActions.svelte`: wrap in `ActionGroup` so reasons render below the
   row and buttons are sized by label.

In Rust, test the chooser's inspection path (`inspect_input`) with a native
bundle's manifest, CSV, and BIN, and a derived bundle's manifest and CSV.
Cover standalone current and legacy files, including a BIN with a valid CSV
sibling, and a flat `_concat_` CSV. Assert the exact selected basename, no
path separators in either field, and a predicted XLSX basename matching the
generated file. Cover directory-only inspection and the Combine generate path
(preview `inputBasename` is `manifest.json`). Keep the existing workbook and
timestamp assertions. In frontend tests, assert both preview rows and check
that ReportActions renders shared reasons once below the group and references
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
   only when `combine.result` exists. Their current "Create a derived bundle
   before …" disabled reasons become unreachable and are removed.
6. Replace dialog stays in `CombineActions` (or moves with the result buttons)
   without behavior change.

Tests: card vs table rendering by container width (browser), DOM order of
step buttons, Remove accessible name in card layout, result buttons absent
before a result.

### Phase 6 — Copy and Help (UX-11)

Files: `src/copy.ts`, `src/copy.audit.test.ts`, `src/state/controls.ts`,
`src/pages/*.svelte`, `src/pages/HelpPage.svelte` (+ test), backend
user-facing strings in `src-tauri/src/combine/mod.rs`,
`src-tauri/src/commands/combine.rs`, `src-tauri/src/commands/reports.rs`,
`src-tauri/src/reports/mod.rs`, `src-tauri/src/coordinator/state.rs`,
`src-tauri/src/dto/error.rs` (+ their Rust tests), all frontend tests
referencing changed strings, `tests/e2e/*.spec.ts`.

1. Apply every row of spec section 5 through `copy` constants; move inline
   page intros (Reports, Combine) and the Collect metric labels into `copy`.
2. Keep `copy.chart.title` and `copy.chart.caption` with "Descriptive"; keep
   `copy.chart.boundary` verbatim.
3. Apply the spec section 5 "Messages" rows to user-facing text that reaches
   the UI from either side: outcome titles and bodies, safe error messages,
   and disabled reasons say "combined file" instead of "derived bundle".
   `openDerivedFolder` becomes "Open combined folder". Do not change
   `kindLabel` values, error codes, manifest kinds, or file-name tokens.
   Before editing, grep frontend and backend for logic that compares message
   text (`===`, `includes`, `contains`); only display strings change, and the
   full Rust and frontend suites must pass unchanged apart from the expected
   message literals (approved condition, spec section 8 item 5).
4. Update Help steps and control names (Create combined file, result card
   placement, Session name, Input/Report file, lock note); technical terms stay
   in "File formats and version details".
5. Extend `copy.audit.test.ts`: chart heading still contains "Descriptive";
   no "provenance-bearing", "same-stem", or "derived bundle" anywhere in
   `copy`. That test only scans the `copy` object, so also add a
   `HelpPage.test.ts` check that those terms appear only inside the "File
   formats and version details" disclosure, and a Rust test that the changed
   outcome and `SafeError` messages no longer contain "derived bundle".
6. Grep tests and Help for old literals and replace with constants.

### Phase 7 — Browser coverage, docs, and validation

1. Using the Phase 0 harness, run the following checks at both 800x600 and
   1280x800 in light and dark.
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
2. Run the full verified command list from `AGENTS.md`, including MSRV and
   `npm run tauri -- build --no-bundle -- --locked`.
3. Capture after-screenshots of every scenario at both sizes/themes and
   compare with the Phase 0 baseline.
4. Update `docs/DECISIONS.md` (amend "Collect layout (2026-09-06)" breakpoint
   and per-column container queries; Reports preview names; Combine step
   layout and narrow cards; neutral Stop; lock note; approved wording
   including backend messages), `docs/PROJECT_CONTEXT.md` evidence, `TODO.md`,
   and set the spec and this plan to Implemented.
5. Ask the user for a native `npm run tauri dev` smoke at the minimum window
   on Windows (PseudoRNG collect/stop, Reports preview and generate, Combine
   two CSVs) and record the result only as reported.

## Risks and mitigations

- **1280x800 regression from the new breakpoint:** measure the actual main
  container width in the native window; tune between 48–56rem and keep the
  browser bounding-box test as the guard.
- **Container-query scope:** nested named containers change which width each
  utility resolves against; the Phase 0 harness measures both sizes before
  and after Phase 2.
- **Accessibility regressions from moving reasons:** every disabled control
  keeps an `aria-describedby` target; tests assert it.
- **DTO change:** update mocks, fixtures, and `src/ipc/client.test.ts` in
  Phase 4 so typed IPC stays consistent.
- **IPC mock drift:** the Phase 0 mock rejects unknown commands and its
  fixtures come from the typed scenarios, so DTO changes fail the browser
  tests instead of passing silently.

## Commit strategy (when authorized)

One commit per phase on the implementation branch, each passing the frontend
suite; Phases 4 and 6 also pass the Rust suite. Push and any pull request
only with explicit approval.

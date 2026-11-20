# RngKit Usability and Visual Polish Design

**Status:** Approved 2026-11-20 with the recommended answers to section 8.
Implementation follows `docs/plans/2026-11-20-rngkit-usability-polish-plan.md`
and still requires separate authorization to start.

**Date:** 2026-11-20

**Relationship to existing designs:** Extends the approved 2026-08-22 baseline,
2026-08-24 workflow improvements, 2026-08-25 artifact feedback, and the
2026-09-06 Collect layout and Help decisions. Once approved, it supersedes only
the conflicting presentation details named below. All recording, statistical,
privacy, IPC-authority, and authorization boundaries remain in force.

## 1. Context

A review on 2026-11-20 exercised every development scenario in the browser
(production components, mock snapshots, no Tauri IPC or hardware) at 1280x800
light and 800x600 dark, and read the corresponding components. Frontend unit
tests (131), `svelte-check`, and ESLint passed. Rust tests, the native window,
and hardware were not exercised.

Functionality is complete and the safety model (disabled-with-reason controls,
Cancel-first Replace, explicit source selection) is sound. The problems are
layout at the declared 800x600 minimum window, one missing piece of input
identification in Reports, one misleading chart status, weak visual distinction
for some actions, and dense or repeated technical copy.

## 2. Goals

1. Make Collect and Combine fully usable at the 800x600 minimum window without
   regressing the 1280x800 "Monitoring fits" decision.
2. Show which file a report is being generated from.
3. Make every visible status message true for the current state.
4. Give actions clear visual hierarchy and stable layout across states.
5. Reduce repeated and technical wording while keeping Z explicitly
   descriptive.

## 3. Non-goals

- New features (sample-count limit, session notes), new IPC commands, new
  capabilities, or changes to file formats, manifests, or XLSX contents.
- Renaming internal types, IPC fields, file-name tokens, or manifest kinds.
- Statistical inference of any kind; p-values, pass/fail, or confidence
  wording remain forbidden.
- Navigation redesign, localization, or new themes.

## 4. Findings and requirements

Each requirement lists the current behavior, the required behavior, and its
acceptance criteria. Priority: **P1** must ship together; **P2** should ship
in the same change set; **P3** is copy work using the approved wording in
section 5.

### UX-1 (P1) Collect layout at narrow widths

**Current.** `src/pages/CollectPage.svelte:17` switches to two columns when the
container reaches `36rem`. At 800x600 the main area is about 624 px, so
Monitoring gets about 220 px: metric values wrap mid-token (`00:` / `00:` /
`12`, `0.5` / `104`), labels wrap (`Descripti` / `cumulativ`), and the chart
plot is about 150 px wide.

**Required.**
- Use one column (Session above Monitoring) until the Collect container is wide
  enough for both columns at readable size; target breakpoint `52rem`, tuned so
  1280x800 keeps two columns.
- Metric values never break inside a value (`whitespace-nowrap`, tabular
  digits). Metric labels may wrap only between words.
- In the stacked layout, Session uses the full width; its fields may use two
  columns where space allows (sample size and interval side by side).

**Acceptance.**
- At 800x600, each metric value renders on one line and the chart plot area is
  at least 480 px wide.
- At 1280x800, the existing two-column layout and the 2026-09-06 "Monitoring
  fits" behavior are unchanged (Browser test compares bounding boxes).
- No horizontal scroll of `main` at 800x600 in any Collect scenario.

### UX-2 (P1) Combine table and summary at narrow widths

**Current.** At 800x600 the input table scrolls horizontally inside its wrapper
and the State and Remove columns are off-screen. Timestamps and `0 - Raw` wrap
inside cells. In `CompatibilitySummary.svelte:14` the monospace bundle stem has
no break rule and overlaps the Inputs value.

**Required.**
- When the Combine container is narrower than the full table, render each input
  as a compact card (ordinal, basename, state, Remove on the first line; format,
  source, bits, interval, fold, time range, rows as a definition list). At
  wider sizes keep the table.
- In the table, fold, timestamps, and numeric cells do not wrap; basename may
  break at any character.
- Summary grid uses `minmax(0, …)` tracks and the stem uses `break-all`.

**Acceptance.**
- At 800x600, every row's state and Remove control are visible without
  horizontal scrolling, and Remove keeps its accessible name
  `Remove <basename> #<ordinal>`.
- Summary fields never overlap at any width from 800 to 1920 px.

### UX-3 (P1) Reports shows the selected input file

**Current.** The preview (`ReportPreview`, `src/ipc/types.ts:121`,
`src-tauri/src/dto/state.rs:156`) shows kind, origin, source, bits, interval,
fold, status, and rows, but not which file was chosen. The backend already
retains the authoritative source basename for generation.

**Required.**
- Add `basename: String` to the `ReportPreview` DTO, filled from the same
  basename the backend treats as authoritative for the same-stem XLSX. It is a
  file name only, never a path, consistent with Combine rows.
- Show it first in the preview as **Input file** (monospace, `break-all`).
- Show the resulting report name as **Report file** (`<stem>.xlsx`), so
  "same-stem" no longer needs explaining in the intro. The name is derived
  from the same backend stem used by generation; it is a file name only.

**Acceptance.**
- Rust DTO/inspection tests assert the basename for native, derived, standalone
  current, standalone legacy, BIN-with-CSV-sibling, and `_concat_` inputs, and
  assert that no path separator appears in it.
- Frontend test asserts the Input file row; mocks and fixtures are updated.

### UX-4 (P1) Chart status line matches state

**Current.** `LiveZChart.svelte` shows `View paused; new samples still append`
(`src/copy.ts:78`) whenever it is not following, including Ready with zero
points and Completed/Failed, when no samples can arrive.

**Required.**
- Collecting/Stopping: keep `Following new samples` / `View paused; new
  samples still append`.
- Not collecting, with points: `Showing the whole session` after Fit all or by
  default; `Zoomed view — use Fit all to see the whole session` after zoom/pan.
- No points: hide the status phrase; show only the empty-state message.

**Acceptance.** Component tests cover idle-empty, collecting following/paused,
completed full/zoomed, and failed; the paused phrase never appears outside
collecting/stopping.

### UX-5 (P2) Stop is visually distinct from Start

**Current.** Start and Stop both use the primary blue
(`CollectionControls.svelte:16,24`).

**Required.** Add a neutral dark `stop` button variant (Stop saves the
session, so it must not suggest data loss) with its own theme tokens
(`--color-action-stop`, `--color-action-stop-hover`, `--color-action-stop-text`)
in the light, dark, and system-dark blocks of `src/styles/theme.css`: light uses
the inverse surface (`#152033`) with inverse text; dark uses a light neutral
(`#e8eef5`) with dark text. Add a square stop glyph marked `aria-hidden`. Label
text stays `Stop`. Start keeps the primary blue.

**Acceptance.** `theme.contrast.test.ts` asserts at least 4.5:1 for stop text on
the stop background in both themes; component test asserts the variant.

### UX-6 (P2) Stable action layout and clearer disabled state

**Current.** `Button.svelte` renders its disabled reason as a paragraph inside a
column wrapper, so a disabled button grows to the width of its reason. This
misaligns Reports actions and makes Refresh sources jump between compact
(Ready) and full width (Collecting). Disabled secondary buttons differ from
enabled ones only by text color.

**Required.**
- Button width depends only on its label. Reasons are rendered by the action
  group, below the group, one line per distinct reason, and stay linked to each
  button through `aria-describedby`.
- When collection locks the Session panel, show one note at the top of Session
  — `Settings are locked while a session is collecting.` (or `…stopping.`) —
  instead of per-control reasons for Refresh, Choose folder, and Open working
  folder. Each locked control references that note via `aria-describedby`.
- Disabled secondary/ghost buttons use a muted background, transparent border,
  muted text, and `cursor-not-allowed`.

**Acceptance.**
- Refresh sources and Device setup keep the same position and size across
  Ready, Collecting, Stopping, and Completed (Browser test).
- Reports action buttons share one row height and are sized by label.
- Existing tests that assert reasons by accessible description still pass;
  tests that assert visible inline reasons are updated.

### UX-7 (P2) Status shown once per context

**Current.** The status appears in the header, in the Monitoring badge, and in
Failed also as the badge detail and again as the Error panel title and body.

**Required.**
- Header status remains (it is the only status on Reports/Combine/Help).
- Monitoring badge shows only the state label, never the error detail.
- Error panel heading becomes `What happened`; message, recovery, and Copy
  diagnostics stay.

**Acceptance.** In the Failed scenario the error message text appears exactly
once on the page; the `role="status"` live region still announces state changes.

### UX-8 (P2) Label the session name

**Current.** `SessionSummary.svelte` prints the session stem as unlabeled
monospace text.

**Required.** Render it as a labeled item `Session name` with the stem as value.

### UX-9 (P2) Combine shows the order of steps

**Current.** Six equal buttons sit in one row (`CombineActions.svelte`).

**Required.** Group actions by step:
1. **Files** — Add files, Clear all (above the table).
2. **Create** — Create derived bundle (primary, below the table, next to the
   compatibility message).
3. **Result** — Generate XLSX and Open folder inside the result summary card,
   shown only after a result exists.

Open working folder becomes a ghost button aligned to the page header.

**Acceptance.** Button order in the DOM follows the steps; Help step text is
updated to match the new placement.

### UX-10 (P2) Chart reference labels stay legible

**Current.** `Reference ±1.96` and `Zero` labels are drawn without a backdrop
and are crossed by the data line.

**Required.** Draw each guide label on a small surface-colored backdrop at the
left of the plot (existing position), scaled by device pixel ratio.

**Acceptance.** Adapter unit test asserts the backdrop is drawn before the text
for each guide label.

### UX-11 (P3) Plainer, non-repeated copy

Applies the wording in section 5. "Descriptive cumulative Z" appears today as
metric label, chart heading, Y-axis title, and legend series. After the change
it remains the chart heading and the accessible caption; the other three
places use shorter forms. The approved boundary sentence `Z shows balance over
time; it does not certify randomness.` is unchanged.

**Acceptance.** `copy.audit.test.ts` still passes; Help references to renamed
labels are updated; e2e and unit tests use `copy` constants, not literals.

## 5. Approved wording

| Location | Current | Proposed |
| --- | --- | --- |
| Metric label | Descriptive cumulative Z | Cumulative Z |
| Metric label | Observed one proportion | Share of 1 bits |
| Chart Y-axis | Descriptive cumulative Z | Z |
| Chart legend series | Descriptive cumulative Z | Cumulative Z |
| Chart heading | Descriptive cumulative Z | *(unchanged)* |
| Chart empty state | No committed samples yet. Every committed point is retained for this session. | No samples yet. The chart starts with the first sample. |
| Reports intro | Inspect a native or derived bundle, current CSV/BIN, or legacy v3 CSV, then generate a same-stem XLSX report. | Choose a session, combined file, or CSV/BIN file. RngKit saves an Excel (XLSX) report next to it. |
| Combine intro | Add compatible current or legacy CSV files from one or more folders and create a provenance-bearing derived bundle without modifying the inputs. | Add CSV files that share the same sample size and interval. RngKit joins them into a new file and never changes the originals. |
| Combine button | Create derived bundle | Create combined file |
| Combine summary | Bundle stem | Combined file name |
| Replace dialog | An XLSX file already exists for this input. Cancel keeps it. Replace is an explicit second request. | A report for this file already exists. Cancel keeps it; Replace overwrites it. |
| Session lock note | *(new)* | Settings are locked while a session is collecting. |
| Error panel heading | Failed | What happened |

Technical names (derived bundle, schema, `_concat_`) remain in Help's "File
formats and version details" section and in documentation.

## 6. Preserved constraints

- Capabilities stay `core:default` and `dialog:default`; CSP unchanged; no new
  IPC command. UX-3 adds one DTO field carrying a basename only.
- No entropy, seeds, serials, device paths, or absolute paths cross IPC beyond
  what the 2026-08-25 outcome-dialog contract already allows.
- Z and ±1.96 remain descriptive visual guides; the boundary sentence is kept.
- 4.5:1 body/muted contrast, reduced-motion support, keyboard order, focus
  return, and `aria-describedby` reasons are preserved.
- Default tests stay hardware-free; locked dependencies; no new packages.

## 7. Validation

- Full verified command list from `AGENTS.md` (frontend, Rust, MSRV,
  no-bundle build).
- New or updated Browser tests at 800x600 and 1280x800 for Collect (all
  collection scenarios), Reports preview/conflict, and Combine
  compatible/incompatible, in light and dark.
- Native `npm run tauri dev` smoke at the minimum window size on Windows by the
  user (PseudoRNG collect, Reports preview, Combine), recorded in
  `docs/PROJECT_CONTEXT.md`.
- On completion: update `docs/DECISIONS.md` (Collect layout amendment, Reports
  basename, Combine step layout, copy), `TODO.md`, and Help.

## 8. Resolved decisions (2026-11-20)

1. **Copy (UX-11):** section 5 is approved as written.
2. **Stop color (UX-5):** neutral dark, not red, because Stop saves the session.
3. **Report file line (UX-3):** show both the input file and the predicted
   `<stem>.xlsx` report name.
4. **Combine at narrow widths (UX-2):** stacked cards below the breakpoint;
   the table remains at wider sizes.

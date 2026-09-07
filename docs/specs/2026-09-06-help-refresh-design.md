# Help copy and presentation refresh

Status: Approved by the user; implementation planning authorized.

## Context and goal

Help currently renders eight topics as a single plain article in
`src/pages/HelpPage.svelte`. Several paragraphs repeat chart behavior or describe
internal implementation rather than user actions. Numeric validation and the
current Session action locations are missing.

Make Help easier to scan, more welcoming, and accurate for the current app.
Preserve English copy, existing topic order, offline availability, and the exact
statistical boundary. This change affects Help presentation and its tests only;
it does not change collection, Reports, Combine, dependencies, or IPC.

## Layout and visual treatment

- Keep the Help page title, followed by one short introduction:
  “Collect samples, create reports, and find answers to common questions.”
- Use a content area up to 64rem wide. At an available content width of 56rem,
  show a 12rem topic-navigation column beside the article; otherwise place the
  navigation above the article as wrapping links. The article remains in one
  column with a readable line length. Do not add another application sidebar.
- Label the navigation “On this page”; link to the eight existing section IDs.
  Links scroll to and focus the target heading without changing app destination.
  Use a visible focus indicator and no animated scrolling.
- Make Quick start a softly accented card with three numbered steps. Use the
  existing theme tokens, border radius, and typography; no images, new fonts,
  gradients, or animation are needed.
- Present remaining topics as separated sections with clear headings, generous
  line height, short paragraphs, and consistent spacing. Bold actual control
  names. Reserve monospace for filenames, paths, and technical identifiers.
- Keep all everyday instructions visible. Use native details/summary disclosures
  only for individual troubleshooting answers and advanced file/version content.
  These start closed and can be opened independently. Keep the file/version
  section heading outside its disclosure so navigation can always reach it.
- Long filenames, revision strings, and error codes wrap without page overflow.
  Use existing light/dark tokens and retain at least 4.5:1 body-text contrast.

## Content contract

Preserve these topics in their current order:

1. **Quick start:** choose one source; configure the session; Start then Stop.
   Mention the new-user defaults (2048 bits and Documents/RngKit), saved settings,
   and choosing another output folder when necessary.
2. **Choosing a source:** automatic discovery, explicit selection, Refresh sources,
   supported source families, existing fold choices, and no silent device switch.
   Keep fold options sourced from FOLD_OPTIONS.
3. **Collecting and stopping safely:** whole bits divisible by 8 (examples 8,
   1024, 2048); whole interval seconds greater than zero (examples 1, 2, 10).
   Explain the correction dialog and that invalid settings do not start collection.
   State that Stop finishes/saves the current sample and describe close choices.
   Locate Open session folder and Start another session in Session.
4. **Creating reports:** choose input, inspect, generate, and explicitly replace
   an existing XLSX when intended. Explain native/derived bundles versus standalone
   CSV/BIN inputs. Keep timestamp versus sample-index behavior in one concise note.
   Describe the result dialog using user actions, without phrases such as
   “backend-confirmed,” “approved action,” or “backend-known state.”
5. **Combining files:** CSV-only, Add files across folders, review/remove/clear,
   matching source/sample size/interval/fold, non-overlapping time ranges, create
   bundle then generate XLSX. State that inputs are unchanged.
6. **Understanding the chart:** preserve exactly “Z shows balance over time; it
   does not certify randomness.” Explain zero and ±1.96 as descriptive guides,
   not pass/fail thresholds. Explain retained points, zoom/pan and Fit all here,
   removing the duplicate explanation from collection instructions.
7. **Common problems:** concise symptom/action disclosures for no sources, invalid
   bits/interval, unavailable output folder, incomplete report bundle, incompatible
   Combine inputs, existing report, locating saved files, and keyboard/display use.
8. **File formats and version details:** retain native BIN/CSV/manifest contents,
   current and legacy CSV conventions, standalone filename metadata, `_concat_`,
   schema-1 compatibility/schema-2 output, application version, pinned library
   revision, and stable error codes in the advanced disclosure.

Proposed incomplete-bundle answer:

> If the folder contains manifest.json, Reports checks the complete bundle.
> A native session needs its matching BIN and CSV files. Restore the missing
> original file, then choose the input again. A standalone BIN is a different
> input type and needs a valid filename; its chart uses sample numbers.

Do not recommend deleting a manifest or renaming metadata to bypass validation.
Keep the complete-bundle rule visible in Creating reports as well as in recovery
help. Validate final wording against current report inspection before implementation.

## Alternatives and trade-offs

- A typography-only cleanup would be smaller but would leave navigation and
  technical information difficult to scan.
- Putting every topic in an accordion would shorten the page but hide the primary
  workflows. Keep those expanded; collapse only troubleshooting answers and reference
  details. The resulting page still scrolls intentionally.

## Files and constraints

Primary surface: `src/pages/HelpPage.svelte`. Keep the implementation local unless
a small Help-only component materially improves readability. Update
`src/pages/HelpPage.test.ts` and relevant existing browser assertions to reflect
visible/disclosed content and remove assertions requiring repeated prose.

Maintain semantic heading order, ordered procedure lists, keyboard-operable
disclosures, and navigation targets. No new persistence, network requests, backend
commands, or app-state changes. Existing application navigation stays unchanged.
Update project context and TODO after implementation and user acceptance.

## Acceptance and validation

- All eight topics and their navigation links remain present and ordered.
- Each topic link reaches its heading using mouse or keyboard; focus remains visible.
- Quick start, numeric rules, Session actions, and bundle requirements are visible
  without opening advanced details. Troubleshooting and advanced disclosures open
  using keyboard controls and reveal their complete content.
- No horizontal overflow or clipped controls at 1280x800 and 800x600, including
  light/dark themes and increased text scale. Scrolling is expected for Help.
- Source/fold labels, revision and error codes remain tied to current constants.
  Statistical wording remains descriptive and inputs remain documented as read-only.
- Run focused Help tests, frontend type checks, lint, format, build, and existing
  browser tests. Verify topic navigation and disclosures in the rendered page.
  Report browser evidence separately from user native acceptance.

## Approval boundary

The user approved this design. Implementation and any later commit/push require
their corresponding authorization.

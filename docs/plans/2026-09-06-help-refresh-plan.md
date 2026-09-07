# Help refresh implementation plan

Status: Implemented; automated/browser checks passed, user native acceptance pending.

## Goal and scope

Implement the approved design in
`docs/specs/2026-09-06-help-refresh-design.md` as one cohesive, independently
testable change. Completion means accurate Help copy, responsive topic navigation,
accessible disclosures, and verified rendering in both themes.

No changes to collection, report generation, Combine behavior, backend commands,
dependencies, persistence, or application navigation. No commit or push is included.

## Prerequisites

- Recheck repository instructions and working-tree changes; preserve unrelated work.
- Use installed, locked dependencies and existing theme tokens.
- Inspect current report inspection code under `src-tauri/src` to confirm the
  manifest/bundle recovery wording. Treat an incomplete bundle and standalone BIN
  as distinct cases; do not suggest removing metadata to bypass validation.

## Ordered implementation steps

### 1. Revise Help content

File: `src/pages/HelpPage.svelte`.

- Retain the eight approved topics, their order, and existing IDs.
- Rewrite Quick start as three short steps, with defaults and saved preferences.
- Add positive whole bits divisible by 8 and positive whole interval seconds,
  the corrective dialog, and terminal button locations in Session.
- Simplify report/Combine result instructions, explain complete bundles and missing
  original files, and retain timestamp provenance and read-only input behavior.
- Consolidate chart controls into Understanding the chart and preserve the exact
  non-certification statement from `copy.chart.boundary`.
- Keep FOLD_OPTIONS, RNGKIT_CORE_REVISION, and ERROR_CODES as source constants.

Acceptance: everyday instructions cover the current controls without developer
jargon; no supported format, recovery rule, or statistical boundary is lost.

### 2. Build the responsive presentation

File: `src/pages/HelpPage.svelte`; use Help-local styles where useful.

- Add the introduction and “On this page” navigation with eight anchor links.
- Use a 64rem maximum content width and container-based 56rem breakpoint:
  12rem navigation beside the article when space permits, wrapping links above
  the article otherwise. Keep the article in one column.
- Make target headings programmatically focusable with `tabindex="-1"`.
  Link activation focuses and scrolls to its heading without smooth motion or
  changing the application destination. Retain visible focus styling.
- Style Quick start as a softly accented numbered card. Apply consistent section
  spacing, readable line height, control-name emphasis, and wrapping technical text.
- Add native details/summary elements for the specified troubleshooting answers
  and one advanced file/version disclosure. Start closed; allow independent opening.
  Keep the file/version heading outside the disclosure.
- Use existing theme tokens and add no assets or dependencies.

Acceptance: workflows stay visible, technical details are discoverable, and all
navigation/disclosure controls are keyboard accessible in both layouts.

### 3. Update focused and browser coverage

Files: `src/pages/HelpPage.test.ts`, `tests/e2e/scaffold.spec.ts`, and
`tests/e2e/accessibility.spec.ts` where the existing assertions need adjustment.

- Verify topic order, navigation targets, current numeric rules, Session actions,
  complete-bundle guidance, source constants and statistical wording.
- Replace assertions that require duplicated prose with checks for the retained
  meaning. Explicitly open advanced details before asserting their visibility.
- Verify keyboard topic activation moves focus to the heading; Enter/Space operate
  native disclosures and reveal the expected content.
- Inspect rendered Help at 1280x800 and 800x600 in light/dark themes and with
  increased text scale. Check wrapping, focus visibility, and horizontal overflow.
- Keep tests hardware-free and distinguish browser state from native evidence.

Acceptance: tests exercise observable navigation and disclosure behavior, not just
the presence of CSS classes or implementation-specific markup.

### 4. Complete verification and context updates

Run:

```text
npm run test:unit -- --run src/pages/HelpPage.test.ts
npm run check
npm run test:e2e
npm run lint
npm run format:check
git diff --check
```

The existing E2E command builds production assets; record that build result.
Run lint after E2E so test-output directory cleanup cannot race its traversal.
Broaden unit testing if shared code changes or a regression warrants it.

Update `docs/PROJECT_CONTEXT.md`, `docs/DECISIONS.md`, and `TODO.md` with the
implemented Help contract, observed checks, and remaining native acceptance.
Review the final diff for scope and truthful product wording.

## User validation gate

Report the result and completed checks, then ask the user to inspect Help in the
native app: follow topic links, open troubleshooting/technical details, switch
themes, and resize to the minimum window. Native acceptance remains pending until
the user reports it. Do not commit or push without separate authorization.

## Risks and safeguards

- Hidden reference text changes existing visibility expectations: update tests to
  open the relevant disclosure instead of weakening assertions.
- Anchor scrolling can leave keyboard focus behind: explicitly focus target headings.
- Long identifiers can overflow narrow layouts: wrap them and verify at minimum size.
- Copy can overstate report support: verify current inspection behavior before editing.

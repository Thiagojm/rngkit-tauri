import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import ReportsPage from '../../pages/ReportsPage.svelte';

describe('ReportActions', () => {
  it('keeps open disabled until a report exists', () => {
    appState.applyScenario('reportsPreview');
    render(ReportsPage);
    expect(
      screen.getByRole('button', { name: copy.openReport }),
    ).toHaveProperty('disabled', true);
  });

  it('renders a shared disabled reason once below the group', () => {
    appState.applyScenario('collecting');
    render(ReportsPage);
    const ids = [
      copy.generateReport,
      copy.openReport,
      copy.openContainingFolder,
      copy.openWorkingFolder,
    ].map((name) =>
      screen.getByRole('button', { name }).getAttribute('aria-describedby'),
    );
    expect(ids[0]).toBeTruthy();
    expect(new Set(ids).size).toBe(1);
    const reason = document.getElementById(ids[0]!)!;
    expect(reason.closest('button')).toBeNull();
    const group = reason.parentElement!;
    expect(
      [...group.querySelectorAll('p')].filter(
        (p) => p.textContent === reason.textContent,
      ),
    ).toHaveLength(1);
  });

  it('links distinct reasons to their own buttons', () => {
    appState.applyScenario('reportsPreview');
    render(ReportsPage);
    const open = screen.getByRole('button', { name: copy.openReport });
    expect(
      document.getElementById(open.getAttribute('aria-describedby')!)
        ?.textContent,
    ).toContain('Open the report after a report is generated.');
  });

  it('opens the Cancel/Replace dialog instead of generating over a conflict', async () => {
    appState.applyScenario('reportsConflict');
    render(ReportsPage);
    await fireEvent.click(
      screen.getByRole('button', { name: copy.generateReport }),
    );
    expect(
      screen.getByRole('heading', { name: copy.replaceTitle }),
    ).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: copy.cancel }));
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(appState.snapshot.reports.reportReady).toBe(true);
  });
});

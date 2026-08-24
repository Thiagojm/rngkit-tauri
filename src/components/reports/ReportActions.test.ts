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

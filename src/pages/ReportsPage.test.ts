import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../copy';
import { appState } from '../state/app-state.svelte';
import ReportsPage from './ReportsPage.svelte';

describe('ReportsPage', () => {
  it('keeps generate disabled without a preview', () => {
    render(ReportsPage);

    expect(
      screen.getByRole('button', { name: copy.generateReport }),
    ).toHaveProperty('disabled', true);
    expect(screen.getByText(copy.noReportInput)).toBeTruthy();
  });

  it('opens the Cancel/Replace dialog for a conflicting report', async () => {
    appState.applyScenario('reportsConflict');
    appState.destination = 'reports';
    render(ReportsPage);

    expect(screen.getByText('Native session')).toBeTruthy();
    expect(screen.getByText(copy.fold.raw)).toBeTruthy();

    await fireEvent.click(
      screen.getByRole('button', { name: copy.generateReport }),
    );

    expect(
      screen.getByRole('heading', { name: copy.replaceTitle }),
    ).toBeTruthy();
    expect(screen.getByRole('dialog').classList.contains('m-auto')).toBe(true);
    const cancel = screen.getByRole('button', { name: copy.cancel });
    expect(cancel).toBeTruthy();
  });
});

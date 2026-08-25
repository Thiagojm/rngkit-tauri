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
    expect(
      screen.getAllByRole('button', { name: copy.chooseReportInput }),
    ).toHaveLength(1);
  });

  it('shows legacy preview metadata', () => {
    appState.applyScenario('reportsPreview');
    appState.snapshot.reports.preview = {
      kindLabel: 'Legacy v3',
      origin: 'CSV only',
      source: 'TrueRNG v1/v2/v3',
      sampleBits: 16,
      intervalSeconds: 1,
      fold: null,
      status: 'Completed',
      rowCount: 1,
      warning: 'Timestamps are recorded in the CSV input.',
      conflict: false,
    };
    render(ReportsPage);
    expect(screen.getByText('Legacy v3')).toBeTruthy();
    expect(screen.getByText('CSV only')).toBeTruthy();
    expect(
      screen.getByText('Timestamps are recorded in the CSV input.'),
    ).toBeTruthy();
  });

  it('shows derived preview metadata', () => {
    appState.applyScenario('reportsPreview');
    appState.snapshot.reports.preview = {
      kindLabel: 'Derived bundle',
      origin: 'Concatenated legacy v3 CSV',
      source: 'TrueRNG v1/v2/v3',
      sampleBits: 16,
      intervalSeconds: 1,
      fold: null,
      status: 'Completed',
      rowCount: 4,
      warning: 'Timestamps are copied from the concatenated inputs.',
      conflict: false,
    };
    render(ReportsPage);
    expect(screen.getByText('Derived bundle')).toBeTruthy();
    expect(screen.getByText('Concatenated legacy v3 CSV')).toBeTruthy();
    expect(
      screen.getByText('Timestamps are copied from the concatenated inputs.'),
    ).toBeTruthy();
  });

  it('shows native preview metadata', () => {
    appState.applyScenario('reportsPreview');
    render(ReportsPage);
    expect(screen.getByText('Native session')).toBeTruthy();
    expect(screen.getByText(copy.fold.raw)).toBeTruthy();
  });

  it('offers a Reports working-folder action without a frontend path', () => {
    render(ReportsPage);

    expect(
      screen.getByRole('button', { name: copy.openWorkingFolder }),
    ).toHaveProperty('disabled', false);
  });

  it('opens the Cancel/Replace dialog for a conflicting report', async () => {
    appState.applyScenario('reportsConflict');
    appState.destination = 'reports';
    render(ReportsPage);

    expect(screen.getByText('Native session')).toBeTruthy();
    expect(screen.getByText(copy.fold.raw)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.openReport }),
    ).toHaveProperty('disabled', false);

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

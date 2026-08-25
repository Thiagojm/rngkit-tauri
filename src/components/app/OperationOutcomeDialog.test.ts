import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import type { OutcomeNotice } from '../../state/types';
import { appState } from '../../state/app-state.svelte';
import OperationOutcomeDialog from './OperationOutcomeDialog.svelte';

const notice: OutcomeNotice = {
  id: 7,
  severity: 'success',
  operation: 'collection',
  title: 'Collection complete',
  message: 'The recording is ready.',
  paths: [
    { label: 'Session folder', path: 'C:\\RngKit\\session' },
    { label: 'Session CSV', path: 'C:\\RngKit\\session\\session.csv' },
  ],
  actions: ['openSessionFolder'],
};

function showNotice(): void {
  appState.applyScenario('completed');
  appState.reconcile({ ...appState.snapshot, pendingOutcome: notice });
}

describe('OperationOutcomeDialog', () => {
  it('renders one notice with selectable paths and approved actions', async () => {
    showNotice();
    render(OperationOutcomeDialog);

    expect(screen.getAllByRole('dialog')).toHaveLength(1);
    expect(screen.getByRole('heading', { name: notice.title })).toBeTruthy();
    expect(screen.getByText(notice.paths[0].path)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: 'Open session folder' }),
    ).toBeTruthy();
  });

  it('acknowledges once on dismiss and does not reopen the same notice', async () => {
    showNotice();
    render(OperationOutcomeDialog);

    await fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));

    expect(appState.snapshot.pendingOutcome).toBeNull();
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});

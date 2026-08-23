import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import CloseCollectionDialog from './CloseCollectionDialog.svelte';

describe('CloseCollectionDialog', () => {
  it('offers Keep collecting and Stop and exit while collecting', async () => {
    appState.applyScenario('collecting');
    appState.onCloseRequested('confirm');
    render(CloseCollectionDialog);

    expect(screen.getByText(copy.close.body)).toBeTruthy();
    await fireEvent.click(
      screen.getByRole('button', { name: copy.close.keep }),
    );
    expect(appState.closePrompt).toBe('none');
  });

  it('Stop and exit requests a cooperative stop', async () => {
    appState.applyScenario('collecting');
    appState.onCloseRequested('confirm');
    render(CloseCollectionDialog);

    await fireEvent.click(
      screen.getByRole('button', { name: copy.close.stopAndExit }),
    );
    expect(appState.snapshot.collection.state).toBe('completed');
  });

  it('shows a finalizing message without a second stop action', () => {
    appState.applyScenario('stopping');
    appState.onCloseRequested('finalizing');
    render(CloseCollectionDialog);

    expect(
      screen.getByRole('heading', { name: copy.close.finalizingTitle }),
    ).toBeTruthy();
    expect(screen.getByText(copy.close.finalizingBody)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.close.stopAndExit }),
    ).toHaveProperty('disabled', true);
    expect(screen.queryByRole('button', { name: copy.close.keep })).toBeNull();
  });
});

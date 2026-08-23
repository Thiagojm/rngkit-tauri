import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import CollectPage from '../../pages/CollectPage.svelte';
import { appState } from '../../state/app-state.svelte';

describe('CollectionControls', () => {
  it('starts a ready draft and then shows Stop', async () => {
    appState.applyScenario('ready');
    render(CollectPage);

    await fireEvent.click(screen.getByRole('button', { name: copy.start }));

    expect(appState.snapshot.collection.state).toBe('collecting');
    expect(screen.queryByRole('button', { name: copy.start })).toBeNull();
    expect(screen.getByRole('button', { name: copy.stop })).toHaveProperty(
      'disabled',
      false,
    );
  });

  it('stops collection into a completed summary', async () => {
    appState.applyScenario('collecting');
    render(CollectPage);

    await fireEvent.click(screen.getByRole('button', { name: copy.stop }));

    expect(appState.snapshot.collection.state).toBe('completed');
    expect(
      screen.getByRole('button', { name: copy.openSessionFolder }),
    ).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.startAnother }),
    ).toBeTruthy();
  });
});

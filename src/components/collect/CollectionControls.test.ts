import { fireEvent, render, screen, within } from '@testing-library/svelte';
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

describe('Session configuration validation', () => {
  it.each(['2049', '0', '-8', '8.5', ''])(
    'blocks invalid sample size %s',
    async (value) => {
      appState.applyScenario('ready');
      render(CollectPage);
      await fireEvent.input(screen.getByLabelText(copy.sampleBits), {
        target: { value },
      });
      await fireEvent.click(screen.getByRole('button', { name: copy.start }));
      expect(appState.snapshot.collection.state).toBe('ready');
      expect(
        screen.getByRole('dialog', { name: 'Cannot start collection' }),
      ).toBeTruthy();
      await fireEvent.click(
        screen.getByRole('button', { name: 'Edit sample size' }),
      );
      expect(document.activeElement).toBe(
        screen.getByLabelText(copy.sampleBits),
      );
      await fireEvent.input(screen.getByLabelText(copy.sampleBits), {
        target: { value: '2048' },
      });
      await fireEvent.click(screen.getByRole('button', { name: copy.start }));
      expect(appState.snapshot.collection.state).toBe('collecting');
    },
  );

  it.each(['0', '-1', '1.5', ''])(
    'blocks invalid interval %s',
    async (value) => {
      appState.applyScenario('ready');
      render(CollectPage);
      await fireEvent.input(screen.getByLabelText(copy.sampleInterval), {
        target: { value },
      });
      await fireEvent.click(screen.getByRole('button', { name: copy.start }));
      expect(appState.snapshot.collection.state).toBe('ready');
      await fireEvent.click(
        screen.getByRole('button', { name: 'Edit sample interval' }),
      );
      expect(document.activeElement).toBe(
        screen.getByLabelText(copy.sampleInterval),
      );
      await fireEvent.input(screen.getByLabelText(copy.sampleInterval), {
        target: { value: '3' },
      });
      await fireEvent.click(screen.getByRole('button', { name: copy.start }));
      expect(appState.snapshot.collection.intervalSeconds).toBe(3);
      expect(appState.snapshot.collection.state).toBe('collecting');
    },
  );

  it('places terminal actions in Session', () => {
    appState.applyScenario('completed');
    render(CollectPage);
    const session = within(screen.getByRole('region', { name: 'Session' }));
    expect(
      session.getByRole('button', { name: copy.openSessionFolder }),
    ).toBeTruthy();
    expect(
      session.getByRole('button', { name: copy.startAnother }),
    ).toBeTruthy();
  });
});

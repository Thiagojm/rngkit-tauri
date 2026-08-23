import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import CollectPage from '../../pages/CollectPage.svelte';

describe('SessionConfiguration', () => {
  it('keeps Start disabled until a valid draft is ready', () => {
    render(CollectPage);

    expect(screen.getByRole('button', { name: copy.start })).toHaveProperty(
      'disabled',
      true,
    );
    expect(screen.getByLabelText(copy.sampleBits)).toBeTruthy();
    expect(screen.getByLabelText(copy.sampleInterval)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.chooseFolder }),
    ).toHaveProperty('disabled', false);
  });

  it('rejects a non-byte-aligned sample size', async () => {
    appState.applyScenario('ready');
    render(CollectPage);

    const bits = screen.getByLabelText(copy.sampleBits);
    await fireEvent.change(bits, { target: { value: '7' } });

    expect(appState.snapshot.collection.sampleBits).toBe(8);
    expect(appState.snapshot.preferencesWarning).toMatch(/multiple of 8/i);
    expect(screen.getByRole('button', { name: copy.start })).toHaveProperty(
      'disabled',
      false,
    );
  });

  it('updates interval and fold on a ready BitBabbler draft', async () => {
    appState.applyScenario('ready');
    render(CollectPage);

    await fireEvent.change(screen.getByLabelText(copy.sampleInterval), {
      target: { value: '2' },
    });
    await fireEvent.change(screen.getByLabelText(copy.fold.label), {
      target: { value: '2' },
    });

    expect(appState.snapshot.collection.intervalSeconds).toBe(2);
    expect(appState.snapshot.collection.fold).toBe(2);
  });

  it('records a chosen folder label in development without a filesystem path', async () => {
    render(CollectPage);

    await fireEvent.click(
      screen.getByRole('button', { name: copy.chooseFolder }),
    );

    expect(appState.snapshot.collection.outputRootLabel).toBe('Chosen folder');
    expect(JSON.stringify(appState.snapshot)).not.toMatch(/[A-Za-z]:\\/);
  });
});

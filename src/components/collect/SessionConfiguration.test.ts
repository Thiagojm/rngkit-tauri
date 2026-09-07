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
    await fireEvent.input(bits, { target: { value: '7' } });

    expect(appState.snapshot.collection.sampleBits).toBe(2048);
    await fireEvent.click(screen.getByRole('button', { name: copy.start }));
    expect(
      screen.getByRole('dialog', { name: 'Cannot start collection' }),
    ).toBeTruthy();
    expect(appState.snapshot.collection.state).toBe('ready');
    expect(screen.getByRole('button', { name: copy.start })).toHaveProperty(
      'disabled',
      false,
    );
  });

  it('updates interval and fold on a ready BitBabbler draft', async () => {
    appState.applyScenario('ready');
    render(CollectPage);

    await fireEvent.input(screen.getByLabelText(copy.sampleInterval), {
      target: { value: '2' },
    });
    await fireEvent.change(screen.getByLabelText(copy.fold.label), {
      target: { value: '2' },
    });

    await fireEvent.click(screen.getByRole('button', { name: copy.start }));
    expect(appState.snapshot.collection.intervalSeconds).toBe(2);
    expect(appState.snapshot.collection.fold).toBe(2);
  });

  it('offers the backend-known Collect working folder after a root is chosen', () => {
    appState.applyScenario('ready');
    render(CollectPage);

    expect(
      screen.getByRole('button', { name: copy.openWorkingFolder }),
    ).toHaveProperty('disabled', false);
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

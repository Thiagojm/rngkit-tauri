import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { appState } from '../state/app-state.svelte';
import { copy } from '../copy';
import CollectPage from './CollectPage.svelte';

describe('CollectPage', () => {
  it('shows a disabled Start control and no Stop in idle', () => {
    render(CollectPage);

    const start = screen.getByRole('button', { name: copy.start });
    expect(start).toHaveProperty('disabled', true);
    expect(screen.getByText(copy.start)).toBeTruthy();
    expect(screen.queryByRole('button', { name: copy.stop })).toBeNull();
    expect(screen.getByText(copy.statsWarning)).toBeTruthy();
  });

  it('shows fold labels and Start when ready', () => {
    appState.applyScenario('ready');
    render(CollectPage);

    expect(screen.getByLabelText(copy.fold.label)).toBeTruthy();
    expect(screen.getByRole('option', { name: copy.fold.raw })).toBeTruthy();
    expect(screen.getByRole('radio', { name: /BitBabbler/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: copy.start })).toHaveProperty(
      'disabled',
      false,
    );
    expect(screen.queryByRole('button', { name: copy.stop })).toBeNull();
  });

  it('updates the selected mock source and conditional fold control', async () => {
    appState.applyScenario('ready');
    render(CollectPage);

    await fireEvent.click(screen.getByRole('radio', { name: /PseudoRNG/ }));

    expect(appState.selectedToken).toBe('mock-pseudo-1');
    expect(screen.getByText(/Selected source: PseudoRNG/)).toBeTruthy();
    expect(screen.queryByLabelText(copy.fold.label)).toBeNull();
  });

  it('associates the output folder label and hint with its control group', () => {
    render(CollectPage);

    const group = screen.getByRole('group', { name: copy.outputRoot });
    const hint = screen.getByText('No output folder selected.');

    expect(group.getAttribute('aria-describedby')).toBe(hint.id);
    expect(screen.getByLabelText(copy.sampleBits)).toBeTruthy();
    expect(screen.getByLabelText(copy.sampleInterval)).toBeTruthy();
  });

  it('shows Stop and not Start while collecting', () => {
    appState.applyScenario('collecting');
    render(CollectPage);

    expect(screen.queryByRole('button', { name: copy.start })).toBeNull();
    expect(screen.getByRole('button', { name: copy.stop })).toHaveProperty(
      'disabled',
      false,
    );
    expect(screen.getByText('+0.72')).toBeTruthy();
    expect(screen.getByText(/Retained points: 12/)).toBeTruthy();
    expect(screen.getByText(copy.chart.refPlus)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.chart.resetView }),
    ).toHaveProperty('disabled', false);
  });

  it('explains that stop is already in progress while stopping', () => {
    appState.applyScenario('stopping');
    render(CollectPage);

    expect(screen.getByRole('button', { name: copy.stop })).toHaveProperty(
      'disabled',
      true,
    );
    expect(screen.getByText(/Stop is already in progress/i)).toBeTruthy();
    expect(screen.queryByRole('button', { name: copy.start })).toBeNull();
  });

  it('shows a recoverable error panel after failure', () => {
    appState.applyScenario('failed');
    render(CollectPage);

    expect(screen.getByTestId('error-panel')).toBeTruthy();
    expect(
      screen.getByText('Select another source and try again.'),
    ).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.errors.copyDiagnostics }),
    ).toHaveProperty('disabled', false);
  });

  it('shows terminal actions after completion', () => {
    appState.applyScenario('completed');
    render(CollectPage);

    expect(
      screen.getByRole('button', { name: copy.openSessionFolder }),
    ).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.startAnother }),
    ).toBeTruthy();
    expect(screen.queryByRole('button', { name: copy.start })).toBeNull();
    expect(screen.queryByRole('button', { name: copy.stop })).toBeNull();
  });
});

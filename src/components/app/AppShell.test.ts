import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import AppShell from './AppShell.svelte';

describe('AppShell', () => {
  it('navigates the four destinations from the primary rail', async () => {
    render(AppShell);

    expect(
      screen.getByRole('heading', { name: copy.destinations.collect }),
    ).toBeTruthy();

    await fireEvent.click(
      screen.getByRole('button', { name: copy.destinations.help }),
    );
    expect(
      screen.getByRole('heading', { name: copy.destinations.help }),
    ).toBeTruthy();
    expect(appState.destination).toBe('help');

    await fireEvent.click(
      screen.getByRole('button', { name: copy.destinations.reports }),
    );
    expect(
      screen.getByRole('heading', { name: copy.destinations.reports }),
    ).toBeTruthy();

    await fireEvent.click(
      screen.getByRole('button', { name: copy.destinations.combine }),
    );
    expect(
      screen.getByRole('heading', { name: copy.destinations.combine }),
    ).toBeTruthy();
  });

  it('opens Device setup from Collect without changing collection state', async () => {
    HTMLElement.prototype.scrollIntoView = () => {};
    appState.applyScenario('collecting');
    render(AppShell);

    expect(appState.snapshot.collection.state).toBe('collecting');
    expect(screen.getByText(/Retained points: 12/)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.chart.fitAll }),
    ).toBeTruthy();

    await fireEvent.click(
      screen.getByRole('button', { name: copy.deviceSetup }),
    );

    const heading = screen.getByRole('heading', { name: copy.deviceSetup });
    await waitFor(() => {
      expect(document.activeElement).toBe(heading);
    });
    expect(appState.destination).toBe('help');
    expect(appState.helpFocusId).toBeNull();
    expect(appState.snapshot.collection.state).toBe('collecting');
    expect(appState.snapshot.collection.sampleCount).toBe(12);

    await fireEvent.click(
      screen.getByRole('button', { name: copy.destinations.collect }),
    );
    expect(appState.destination).toBe('collect');
    expect(appState.snapshot.collection.state).toBe('collecting');
    expect(screen.getByText(/Retained points: 12/)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.chart.fitAll }),
    ).toBeTruthy();
  });

  it('does not jump to Device setup during ordinary Help navigation', async () => {
    HTMLElement.prototype.scrollIntoView = () => {};
    render(AppShell);

    await fireEvent.click(
      screen.getByRole('button', { name: copy.destinations.help }),
    );
    expect(appState.destination).toBe('help');
    expect(appState.helpFocusId).toBeNull();
    expect(document.activeElement).not.toBe(
      screen.getByRole('heading', { name: copy.deviceSetup }),
    );
  });

  it('applies the selected theme to the document', async () => {
    render(AppShell);

    await fireEvent.change(screen.getByLabelText(copy.theme.legend), {
      target: { value: 'dark' },
    });
    expect(document.documentElement.dataset.theme).toBe('dark');

    await fireEvent.change(screen.getByLabelText(copy.theme.legend), {
      target: { value: 'light' },
    });
    expect(document.documentElement.dataset.theme).toBe('light');
  });

  it('exposes a skip link and live status', () => {
    render(AppShell);

    expect(
      screen.getByRole('link', { name: copy.skipToMain }).getAttribute('href'),
    ).toBe('#main-content');
    expect(screen.getByText(`${copy.status}: Idle`)).toBeTruthy();
  });

  it('exposes the development scenario switch in unit tests', async () => {
    render(AppShell);

    const select = screen.getByTestId('dev-scenario-switch');
    await fireEvent.change(select, { target: { value: 'ready' } });
    expect(appState.scenarioId).toBe('ready');
    expect(screen.getByRole('radio', { name: /BitBabbler/ })).toBeTruthy();
  });
});

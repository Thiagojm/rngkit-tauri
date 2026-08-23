import { fireEvent, render, screen } from '@testing-library/svelte';
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

  it('exposes the development scenario switch in unit tests', async () => {
    render(AppShell);

    const select = screen.getByTestId('dev-scenario-switch');
    await fireEvent.change(select, { target: { value: 'ready' } });
    expect(appState.scenarioId).toBe('ready');
    expect(screen.getByRole('radio', { name: /BitBabbler/ })).toBeTruthy();
  });
});

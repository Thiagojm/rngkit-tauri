import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import CollectPage from '../../pages/CollectPage.svelte';

describe('LiveZChart', () => {
  it('shows empty guidance and a disabled Fit all before samples arrive', () => {
    render(CollectPage);

    expect(
      screen.getByRole('heading', { name: copy.chart.title }),
    ).toBeTruthy();
    expect(screen.getByText(copy.chart.empty)).toBeTruthy();
    expect(screen.getByText(copy.chart.refPlus)).toBeTruthy();
    expect(screen.getByText(copy.chart.refMinus)).toBeTruthy();
    expect(screen.getByText(copy.chart.zero)).toBeTruthy();
    expect(screen.getByLabelText(copy.chart.caption)).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.chart.fitAll }),
    ).toHaveProperty('disabled', true);
    expect(screen.getByText(copy.chart.boundary)).toBeTruthy();
    expect(
      screen.queryByRole('button', { name: /Reset view|Return to live/i }),
    ).toBeNull();
  });

  it('keeps Fit all singular while points are retained', async () => {
    appState.applyScenario('collecting');
    render(CollectPage);

    expect(screen.getByText(/Retained points: 12/)).toBeTruthy();
    const status = screen.getByTestId('chart-point-count');
    expect(status.textContent).toContain(copy.chart.following);
    const fitAll = screen.getByRole('button', { name: copy.chart.fitAll });
    expect(fitAll).toHaveProperty('disabled', false);
    expect(
      screen.getAllByRole('button', { name: copy.chart.fitAll }),
    ).toHaveLength(1);

    await fireEvent.click(fitAll);
    expect(status.textContent).toContain(copy.chart.following);
    expect(screen.getByText(/Retained points: 12/)).toBeTruthy();
  });

  it('preserves the mounted plot when the theme changes', async () => {
    appState.applyScenario('collecting');
    const { container } = render(CollectPage);
    const plot = container.querySelector('.uplot');

    appState.setTheme('dark');
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(container.querySelector('.uplot')).toBe(plot);
  });
});

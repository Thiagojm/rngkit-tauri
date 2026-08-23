import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import CollectPage from '../../pages/CollectPage.svelte';

describe('LiveZChart', () => {
  it('shows empty guidance and disabled view controls before samples arrive', () => {
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
      screen.getByRole('button', { name: copy.chart.resetView }),
    ).toHaveProperty('disabled', true);
    expect(
      screen.getByRole('button', { name: copy.chart.returnToLive }),
    ).toHaveProperty('disabled', true);
    expect(
      screen.queryByText(
        /p-value|statistically significant|confidence interval/i,
      ),
    ).toBeNull();
  });

  it('keeps Reset view and Return to live distinct while points are retained', async () => {
    appState.applyScenario('collecting');
    render(CollectPage);

    expect(screen.getByText(/Retained points: 12/)).toBeTruthy();
    const status = screen.getByTestId('chart-point-count');
    expect(status.textContent).toContain(copy.chart.live);
    const reset = screen.getByRole('button', { name: copy.chart.resetView });
    const follow = screen.getByRole('button', {
      name: copy.chart.returnToLive,
    });
    expect(reset).toHaveProperty('disabled', false);
    expect(follow).toHaveProperty('disabled', true);

    await fireEvent.click(reset);
    expect(status.textContent).toContain(copy.chart.paused);
    expect(follow).toHaveProperty('disabled', false);

    await fireEvent.click(follow);
    expect(status.textContent).toContain(copy.chart.live);
    expect(follow).toHaveProperty('disabled', true);
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

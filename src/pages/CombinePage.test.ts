import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../copy';
import { appState } from '../state/app-state.svelte';
import CombinePage from './CombinePage.svelte';

describe('CombinePage', () => {
  it('lists compatible inputs by basename and enables creation', () => {
    appState.applyScenario('combineCompatible');
    render(CombinePage);

    expect(
      screen.getByRole('button', { name: copy.chooseCsvInputs }),
    ).toBeTruthy();
    expect(screen.getByText('20260101T010000_bitb_s8_i1.csv')).toBeTruthy();
    expect(screen.getByText('Current CSV')).toBeTruthy();
    expect(screen.getByText('Legacy v3 CSV')).toBeTruthy();
    expect(
      screen.getByRole('button', {
        name: 'Remove 20260101T010000_bitb_s8_i1.csv #1',
      }),
    ).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.clearCombineInputs }),
    ).toHaveProperty('disabled', false);
    expect(
      screen.getByRole('button', { name: copy.createDerived }),
    ).toHaveProperty('disabled', false);
    expect(
      screen.getByText('20260822T120000_concat_bitb_s8_i1_f0'),
    ).toBeTruthy();
  });

  it('identifies incompatible overlapping inputs', () => {
    appState.applyScenario('combineIncompatible');
    render(CombinePage);

    expect(
      screen.getByRole('button', { name: copy.createDerived }),
    ).toHaveProperty('disabled', true);
    expect(screen.getByRole('status').textContent).toMatch(
      /Overlapping timestamp ranges/i,
    );
  });

  it('offers a Combine working-folder action without a frontend path', () => {
    render(CombinePage);

    expect(
      screen.getByRole('button', { name: copy.openWorkingFolder }),
    ).toHaveProperty('disabled', false);
  });
});

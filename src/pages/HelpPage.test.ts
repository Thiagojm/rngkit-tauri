import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../copy';
import { RNGKIT_CORE_REVISION } from '../library-revision';
import HelpPage from './HelpPage.svelte';

describe('HelpPage', () => {
  it('documents folds, statistical limits, and versions', () => {
    render(HelpPage);

    expect(
      screen.getByRole('heading', { name: copy.destinations.help }),
    ).toBeTruthy();
    expect(screen.getByText(copy.fold.raw)).toBeTruthy();
    expect(screen.getByText(copy.statsWarning)).toBeTruthy();
    expect(screen.getByText(RNGKIT_CORE_REVISION)).toBeTruthy();
    expect(screen.getByText(/one explicitly selected/i)).toBeTruthy();
    expect(
      screen.getByText(/Legacy v3 BIN and CSV files stay read-only/i),
    ).toBeTruthy();
    expect(screen.getByText(/Derived inputs remain unavailable/i)).toBeTruthy();
  });
});

import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../copy';
import { RNGKIT_CORE_REVISION } from '../library-revision';
import HelpPage from './HelpPage.svelte';

describe('HelpPage', () => {
  it('presents the approved task-oriented workflow', () => {
    render(HelpPage);

    expect(
      screen.getByRole('heading', { name: copy.destinations.help }),
    ).toBeTruthy();
    for (const heading of [
      'Quick start',
      'Choosing a source',
      'Collecting and stopping safely',
      'Creating reports',
      'Combining files',
      'Understanding the chart',
      'Common problems',
      'File formats and version details',
    ]) {
      expect(screen.getByRole('heading', { name: heading })).toBeTruthy();
    }
    expect(screen.getByText(copy.fold.raw)).toBeTruthy();
    expect(
      screen.getByText(
        /Z shows balance over time; it does not certify randomness/,
      ),
    ).toBeTruthy();
    expect(screen.getByText(RNGKIT_CORE_REVISION)).toBeTruthy();
    expect(screen.getByText(/Nothing is selected automatically/i)).toBeTruthy();
    expect(screen.getByText(/Inputs are read-only/i)).toBeTruthy();
    expect(screen.getByText(/new bundles use schema 2/i)).toBeTruthy();
    expect(
      screen.getByText(/all actions are available from the keyboard/i),
    ).toBeTruthy();
    expect(screen.queryByText(/p-value/i)).toBeNull();
  });
});

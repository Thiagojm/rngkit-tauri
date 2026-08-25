import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../copy';
import { ERROR_CODES } from '../ipc/types';
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
    expect(
      screen.getByText(
        /searches for sources automatically when the app opens/i,
      ),
    ).toBeTruthy();
    expect(screen.getByText(/Inputs are read-only/i)).toBeTruthy();
    expect(screen.getByText(/new Combine output uses schema 2/i)).toBeTruthy();
    expect(screen.getByText(/YYYYMMDDTHHMMSS/)).toBeTruthy();
    expect(
      screen.getAllByText(/BIN-only reports use sample numbers/i),
    ).toHaveLength(2);
    expect(screen.getByText(/outcome dialog appears once/i)).toBeTruthy();
    expect(
      screen.getByText(/Open working folder in Collect, Reports, or Combine/i),
    ).toBeTruthy();
    expect(
      screen.getByText(/canonical flat legacy concatenation CSV/i),
    ).toBeTruthy();
    expect(screen.queryByText(/estimated timestamps/i)).toBeNull();
    for (const code of ERROR_CODES) {
      expect(screen.getByText(code)).toBeTruthy();
    }
    expect(
      screen.getByText(/all actions are available from the keyboard/i),
    ).toBeTruthy();
    expect(screen.queryByText(/p-value/i)).toBeNull();
    expect(screen.queryByText(/desktop side/i)).toBeNull();
    expect(screen.queryByText(/authoritative/i)).toBeNull();
  });
});

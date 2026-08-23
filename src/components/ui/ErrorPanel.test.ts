import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import ErrorPanel from './ErrorPanel.svelte';

describe('ErrorPanel', () => {
  it('is hidden while idle', () => {
    render(ErrorPanel);
    expect(screen.queryByTestId('error-panel')).toBeNull();
  });

  it('shows a recovery action and copies sanitized diagnostics', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    });
    appState.applyScenario('failed');
    render(ErrorPanel);

    expect(screen.getByTestId('error-panel')).toBeTruthy();
    expect(
      screen.getByText('The selected source became unavailable.'),
    ).toBeTruthy();
    expect(
      screen.getByText('Select another source and try again.'),
    ).toBeTruthy();

    await fireEvent.click(
      screen.getByRole('button', { name: copy.errors.copyDiagnostics }),
    );

    expect(writeText).toHaveBeenCalled();
    const text = String(writeText.mock.calls[0]?.[0] ?? '');
    expect(text.toLowerCase()).not.toMatch(/entropy|seed|serial|selector/);
    expect(text).not.toMatch(/[A-Za-z]:\\/);
    expect(text).not.toMatch(/\/dev\//);
    expect(text).toContain('op-1');
    expect(screen.getByText(copy.errors.copied)).toBeTruthy();
  });
});

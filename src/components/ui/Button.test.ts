import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ButtonHarness from './Button.harness.svelte';

describe('Button', () => {
  it('exposes a visible reason when disabled', () => {
    render(ButtonHarness, {
      disabled: true,
      disabledReason: 'Select a source first.',
      label: 'Start',
    });

    const button = screen.getByRole('button', { name: 'Start' });
    expect(button).toHaveProperty('disabled', true);
    expect(screen.getByText('Select a source first.')).toBeTruthy();
    expect(button.getAttribute('aria-describedby')).toBeTruthy();
  });

  it('points at an external reason instead of rendering one inline', () => {
    render(ButtonHarness, {
      disabled: true,
      disabledReason: 'Inline reason.',
      describedBy: 'shared-note',
      label: 'Refresh',
    });

    const button = screen.getByRole('button', { name: 'Refresh' });
    expect(button.getAttribute('aria-describedby')).toBe('shared-note');
    expect(screen.queryByText('Inline reason.')).toBeNull();
  });

  it('renders the neutral stop variant', () => {
    render(ButtonHarness, { variant: 'stop', label: 'Stop' });

    const button = screen.getByRole('button', { name: 'Stop' });
    expect(button.className).toContain('bg-action-stop');
    expect(button.className).not.toContain('bg-action ');
  });
});

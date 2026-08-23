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
  });
});

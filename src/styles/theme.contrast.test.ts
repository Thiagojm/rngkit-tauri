import { describe, expect, it } from 'vitest';

function channel(value: number): number {
  const scaled = value / 255;
  return scaled <= 0.03928 ? scaled / 12.92 : ((scaled + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
  const raw = hex.replace('#', '');
  const r = Number.parseInt(raw.slice(0, 2), 16);
  const g = Number.parseInt(raw.slice(2, 4), 16);
  const b = Number.parseInt(raw.slice(4, 6), 16);
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrast(foreground: string, background: string): number {
  const a = luminance(foreground);
  const b = luminance(background);
  const lighter = Math.max(a, b);
  const darker = Math.min(a, b);
  return (lighter + 0.05) / (darker + 0.05);
}

describe('theme contrast', () => {
  it('meets 4.5:1 for body and muted text in light and dark palettes', () => {
    expect(contrast('#152033', '#f4f7fb')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#4a5a70', '#f4f7fb')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#f4f7fb', '#1d6fd8')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#1b7f46', '#f4f7fb')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#1d6fd8', '#f4f7fb')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#b42318', '#f4f7fb')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#e8eef5', '#101826')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#9aabc0', '#101826')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#101826', '#6ea8f0')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#3dd68c', '#101826')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#6ea8f0', '#101826')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#f97066', '#101826')).toBeGreaterThanOrEqual(4.5);
  });
});

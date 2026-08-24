import { describe, expect, it } from 'vitest';
import { copy } from './copy';

function flatten(value: unknown): string[] {
  if (typeof value === 'string') {
    return [value];
  }
  if (Array.isArray(value)) {
    return value.flatMap(flatten);
  }
  if (value && typeof value === 'object') {
    return Object.values(value).flatMap(flatten);
  }
  return [];
}

describe('user-facing copy', () => {
  it('keeps Z descriptive and omits secrets or selectors', () => {
    const text = flatten(copy).join('\n');
    expect(text).toMatch(
      /Z shows balance over time; it does not certify randomness/i,
    );
    expect(text).not.toMatch(/p-value/i);
    expect(text).not.toMatch(/statistically significant/i);
    expect(text).not.toMatch(/confidence interval/i);
    expect(text).not.toMatch(/Available after the first committed sample/i);
    expect(text).not.toMatch(/Cumulative Z and the chart lines at ±1\.96/i);
    expect(text).not.toMatch(/entropy byte/i);
    expect(text).not.toMatch(/\bseed=/i);
    expect(text).not.toMatch(/\bserial=/i);
    expect(text).not.toMatch(/COM\d/i);
    expect(text).not.toMatch(/[A-Za-z]:\\/);
    expect(text).not.toMatch(/\/dev\//);
  });
});

import { describe, expect, it } from 'vitest';
import { RNGKIT_CORE_REVISION } from './library-revision';

describe('library revision pin', () => {
  it('is the Gate A rngkit-core commit', () => {
    expect(RNGKIT_CORE_REVISION).toBe(
      '2cdf311dd206cb5e7320ee520ef1e7a5139cc146',
    );
  });
});

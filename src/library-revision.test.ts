import { describe, expect, it } from 'vitest';
import { RNGKIT_CORE_REVISION } from './library-revision';

describe('library revision pin', () => {
  it('is the current reachable rngkit-core commit', () => {
    expect(RNGKIT_CORE_REVISION).toBe(
      '495c3f5acdb6960f90e662927e1466aebae7cffd',
    );
  });
});

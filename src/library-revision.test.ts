import { describe, expect, it } from 'vitest';
import { RNGKIT_CORE_REVISION } from './library-revision';

describe('library revision pin', () => {
  it('is the Gate A rngkit-core commit', () => {
    expect(RNGKIT_CORE_REVISION).toBe(
      '183f3c7811f5593b3b42c2558ac726552b86687d',
    );
  });
});

import { describe, expect, it } from 'vitest';
import { RNGKIT_CORE_REVISION } from './library-revision';

describe('library revision pin', () => {
  it('is the Gate A rngkit-core commit', () => {
    expect(RNGKIT_CORE_REVISION).toBe(
      '3f327e9e88679c26683323f116cd6d7b3ea64fff',
    );
  });
});

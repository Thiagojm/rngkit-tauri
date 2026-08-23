import { describe, expect, it } from 'vitest';
import { MOCK_SCENARIOS, SCENARIO_IDS } from './mock-scenarios';

const serialized = JSON.stringify(MOCK_SCENARIOS);

describe('mock scenarios', () => {
  it('covers every collection state', () => {
    const states = new Set(
      SCENARIO_IDS.map((id) => MOCK_SCENARIOS[id].collection.state),
    );
    expect([...states].sort()).toEqual(
      [
        'idle',
        'discovering',
        'ready',
        'collecting',
        'stopping',
        'completed',
        'failed',
      ].sort(),
    );
  });

  it('does not include entropy, seeds, serials, or OS paths', () => {
    expect(serialized).not.toMatch(/COM\d/i);
    expect(serialized).not.toMatch(/\/dev\//);
    expect(serialized).not.toMatch(/[A-Za-z]:\\/);
    expect(serialized.toLowerCase()).not.toMatch(
      /entropy byte|seed|serial|selector/,
    );
  });

  it('uses basenames rather than absolute input paths', () => {
    for (const row of MOCK_SCENARIOS.combineCompatible.combine.inputs) {
      expect(row.basename).not.toMatch(/[/\\]/);
    }
  });

  it('tracks session identity only on live snapshots', () => {
    expect(MOCK_SCENARIOS.idle.collection.sessionId).toBeNull();
    expect(MOCK_SCENARIOS.collecting.collection.sessionId).toBe('s1');
    expect(MOCK_SCENARIOS.collecting.collection.lastEventSequence).toBe(12);
  });

  it('labels mock candidates with a stable source id', () => {
    const [bitb, pseudo] = MOCK_SCENARIOS.ready.collection.candidates;
    expect(bitb.sourceId).toBe('bitb');
    expect(pseudo.sourceId).toBe('pseudo');
  });
});

import { describe, expect, it } from 'vitest';
import type { CollectionEvent } from './types';
import { AppViewState } from './app-state.svelte';
import { MOCK_SCENARIOS } from './mock-scenarios';

function committed(
  partial: Partial<Extract<CollectionEvent, { kind: 'sampleCommitted' }>> &
    Pick<
      Extract<CollectionEvent, { kind: 'sampleCommitted' }>,
      'sequence' | 'sampleIndex' | 'cumulativeZ'
    >,
): CollectionEvent {
  return {
    kind: 'sampleCommitted',
    sessionId: 's1',
    sampleCount: partial.sampleIndex,
    elapsedLabel: '00:00:01',
    onesProportionLabel: '0.5000',
    cumulativeZLabel: '+0.50',
    ...partial,
  };
}

describe('AppViewState chart retention', () => {
  it('retains one point per accepted sample and ignores stale or duplicate events', () => {
    const state = new AppViewState();
    state.reconcile({
      ...MOCK_SCENARIOS.ready,
      collection: {
        ...MOCK_SCENARIOS.ready.collection,
        state: 'collecting',
        statusLabel: 'Collecting',
        sessionId: 's1',
        lastEventSequence: 0,
      },
    });

    state.acceptCollectionEvent(
      committed({ sequence: 1, sampleIndex: 1, cumulativeZ: 0.1 }),
    );
    state.acceptCollectionEvent(
      committed({ sequence: 2, sampleIndex: 2, cumulativeZ: 0.2 }),
    );
    state.acceptCollectionEvent(
      committed({ sequence: 2, sampleIndex: 2, cumulativeZ: 9.9 }),
    );
    state.acceptCollectionEvent(
      committed({
        sequence: 99,
        sampleIndex: 99,
        cumulativeZ: 9.9,
        sessionId: 'other',
      }),
    );

    expect(state.chartSeries.length).toBe(2);
    expect(state.chartSeries.sampleIndex).toEqual([1, 2]);
    expect(state.chartSeries.cumulativeZ).toEqual([0.1, 0.2]);
    expect(JSON.stringify(state.chartSeries.aligned())).not.toMatch(/entropy/i);
  });

  it('seeds mock collecting snapshots and clears on a new session', () => {
    const state = new AppViewState();
    state.applyScenario('collecting');
    expect(state.chartSeries.length).toBe(12);
    expect(state.chartSeries.sampleIndex[0]).toBe(1);
    expect(state.chartSeries.sampleIndex[11]).toBe(12);

    state.applyScenario('completed');
    expect(state.chartSeries.length).toBe(12);
    state.startAnotherSession();
    expect(state.chartSeries.length).toBe(0);
  });

  it('does not add points for non-sample events', () => {
    const state = new AppViewState();
    state.applyScenario('collecting');
    const before = state.chartSeries.length;
    state.acceptCollectionEvent({
      kind: 'timingOverrun',
      sessionId: 's1',
      sequence: 13,
      overrunCount: 1,
    });
    expect(state.chartSeries.length).toBe(before);
  });
});

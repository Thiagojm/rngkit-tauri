import { describe, expect, it } from 'vitest';
import { AppViewState } from './app-state.svelte';
import { MOCK_SCENARIOS } from './mock-scenarios';

describe('AppViewState close and terminal recovery', () => {
  it('ignores late collection events after a failed snapshot', () => {
    const state = new AppViewState();
    state.reconcile(MOCK_SCENARIOS.failed);
    state.acceptCollectionEvent({
      kind: 'cleanStop',
      sessionId: 's1',
      sequence: 99,
      sampleCount: 99,
      overrunCount: 0,
    });
    expect(state.snapshot.collection.state).toBe('failed');
    expect(state.snapshot.collection.sampleCount).toBe(12);
  });

  it('reconciles a failed snapshot after channel loss', () => {
    const state = new AppViewState();
    state.reconcile(MOCK_SCENARIOS.collecting);
    state.collectionChannelGeneration += 1;
    state.reconcile(MOCK_SCENARIOS.failed);
    expect(state.snapshot.collection.state).toBe('failed');
    expect(state.snapshot.collection.errorRecovery).toBe(
      'Select another source and try again.',
    );
    expect(state.snapshot.diagnostics).toHaveLength(1);
    expect(
      JSON.stringify(state.snapshot.diagnostics).toLowerCase(),
    ).not.toMatch(/entropy|seed|serial|selector/);
  });

  it('Keep collecting dismisses a confirm prompt', () => {
    const state = new AppViewState();
    state.reconcile(MOCK_SCENARIOS.collecting);
    state.onCloseRequested('confirm');
    expect(state.closePrompt).toBe('confirm');
    state.keepCollecting();
    expect(state.closePrompt).toBe('none');
    expect(state.snapshot.collection.state).toBe('collecting');
  });
});

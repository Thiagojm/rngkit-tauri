import { describe, expect, it } from 'vitest';
import { deriveControls } from './controls';
import { MOCK_SCENARIOS, SCENARIO_IDS } from './mock-scenarios';
import { COLLECTION_STATES } from './types';

describe('deriveControls', () => {
  it('never shows Start and Stop together', () => {
    for (const id of SCENARIO_IDS) {
      const controls = deriveControls(MOCK_SCENARIOS[id]);
      expect(controls.showStart && controls.showStop).toBe(false);
    }
  });

  it('enables Start only in ready and Stop only in collecting', () => {
    const byState = Object.fromEntries(
      COLLECTION_STATES.map((state) => {
        const id = SCENARIO_IDS.find(
          (scenario) => MOCK_SCENARIOS[scenario].collection.state === state,
        );
        return [state, id ? deriveControls(MOCK_SCENARIOS[id]) : null];
      }),
    );

    expect(byState.ready?.start.enabled).toBe(true);
    expect(byState.ready?.showStart).toBe(true);
    expect(byState.collecting?.stop.enabled).toBe(true);
    expect(byState.collecting?.showStop).toBe(true);
    expect(byState.stopping?.stop.enabled).toBe(false);
    expect(byState.stopping?.stop.reason).toMatch(/already in progress/i);
    expect(byState.idle?.start.enabled).toBe(false);
    expect(byState.completed?.showTerminalActions).toBe(true);
    expect(byState.failed?.showTerminalActions).toBe(true);
  });

  it('blocks file jobs while collecting or stopping', () => {
    const collecting = deriveControls(MOCK_SCENARIOS.collecting);
    expect(collecting.reports.enabled).toBe(false);
    expect(collecting.combine.enabled).toBe(false);
    expect(collecting.configure.enabled).toBe(false);
  });

  it('shows fold when the selected source requires it', () => {
    expect(deriveControls(MOCK_SCENARIOS.ready).showFold).toBe(true);
    expect(deriveControls(MOCK_SCENARIOS.idle).showFold).toBe(false);
  });
});

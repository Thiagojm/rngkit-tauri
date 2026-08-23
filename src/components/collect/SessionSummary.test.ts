import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import CollectPage from '../../pages/CollectPage.svelte';
import { appState } from '../../state/app-state.svelte';

describe('SessionSummary', () => {
  it('returns to a ready draft from a completed session', async () => {
    appState.applyScenario('completed');
    render(CollectPage);

    expect(screen.getByText('20260822T101500_bitb_s8_i1_f0')).toBeTruthy();
    await fireEvent.click(
      screen.getByRole('button', { name: copy.startAnother }),
    );

    expect(appState.snapshot.collection.state).toBe('ready');
    expect(appState.snapshot.collection.sessionId).toBeNull();
    expect(appState.snapshot.collection.sessionStem).toBeNull();
    expect(screen.getByRole('button', { name: copy.start })).toHaveProperty(
      'disabled',
      false,
    );
  });

  it('ignores stale collection events from another session', () => {
    appState.applyScenario('collecting');
    appState.acceptCollectionEvent({
      kind: 'sampleCommitted',
      sessionId: 's-other',
      sequence: 99,
      sampleIndex: 99,
      sampleCount: 99,
      elapsedLabel: '01:00:00',
      onesProportionLabel: '0.9999',
      cumulativeZLabel: '+9.99',
    });

    expect(appState.snapshot.collection.sampleCount).toBe(12);
    expect(appState.snapshot.collection.lastEventSequence).toBe(12);
  });
});

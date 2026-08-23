import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import { copy } from '../../copy';
import { appState } from '../../state/app-state.svelte';
import { MOCK_SCENARIOS } from '../../state/mock-scenarios';
import CollectPage from '../../pages/CollectPage.svelte';

describe('SourceDiscovery', () => {
  afterEach(() => {
    delete (globalThis as { isTauri?: boolean }).isTauri;
    clearMocks();
  });

  it('shows empty-discovery guidance and does not auto-select', () => {
    render(CollectPage);

    expect(screen.getByText(copy.noSources)).toBeTruthy();
    expect(screen.queryByRole('radio')).toBeNull();
    expect(
      screen.getByRole('button', { name: copy.refreshSources }),
    ).toHaveProperty('disabled', false);
  });

  it('lists multiple mock devices as separate choices', () => {
    appState.applyScenario('ready');
    render(CollectPage);

    const radios = screen.getAllByRole('radio');
    expect(radios).toHaveLength(2);
    expect(
      screen.getByRole('radio', { name: 'BitBabbler · White · 1' }),
    ).toBeTruthy();
    expect(screen.getByRole('radio', { name: 'PseudoRNG · 1' })).toBeTruthy();
  });

  it('surfaces a family warning without hiding candidates', () => {
    appState.applyScenario('ready');
    appState.backendSnapshot = {
      ...appState.backendSnapshot,
      collection: {
        ...appState.backendSnapshot.collection,
        familyWarning:
          'BitBabbler discovery reported a problem (permission was denied). Other sources remain selectable.',
      },
    };
    render(CollectPage);

    expect(screen.getByRole('note').textContent).toMatch(/BitBabbler/);
    expect(screen.getAllByRole('radio')).toHaveLength(2);
    expect(screen.getByRole('note').textContent).not.toMatch(/COM\d|serial/i);
  });

  it('refresh in the browser lists mock candidates without selecting one', async () => {
    render(CollectPage);

    await fireEvent.click(
      screen.getByRole('button', { name: copy.refreshSources }),
    );

    await waitFor(() => {
      expect(screen.getAllByRole('radio')).toHaveLength(2);
    });
    expect(appState.selectedToken).toBeNull();
    expect(screen.queryByRole('radio', { name: /BitBabbler/ })).toHaveProperty(
      'checked',
      false,
    );
    expect(screen.queryByRole('radio', { name: /PseudoRNG/ })).toHaveProperty(
      'checked',
      false,
    );
  });

  it('recovers from a rejected native refresh without staying discovering', async () => {
    (globalThis as { isTauri?: boolean }).isTauri = true;
    mockIPC((command) => {
      if (command === 'refresh_sources') {
        throw {
          code: 'unexpected_failure',
          message: 'The operation failed unexpectedly.',
        };
      }
      if (command === 'get_app_state') {
        return structuredClone(MOCK_SCENARIOS.idle);
      }
      throw new Error(`unexpected command: ${command}`);
    });
    render(CollectPage);

    await fireEvent.click(
      screen.getByRole('button', { name: copy.refreshSources }),
    );

    await waitFor(() => {
      expect(appState.snapshot.collection.state).toBe('idle');
      expect(screen.getByRole('note').textContent).toBe(
        'The operation failed unexpectedly.',
      );
      expect(
        screen.getByRole('button', { name: copy.refreshSources }),
      ).toHaveProperty('disabled', false);
    });
  });

  it('reconciles after a rejected native selection', async () => {
    appState.applyScenario('ready');
    (globalThis as { isTauri?: boolean }).isTauri = true;
    mockIPC((command) => {
      if (command === 'select_source') {
        throw {
          code: 'expired_selection',
          message: 'That source is no longer valid.',
          recovery: 'Refresh sources and select again.',
        };
      }
      if (command === 'get_app_state') {
        return structuredClone(MOCK_SCENARIOS.ready);
      }
      throw new Error(`unexpected command: ${command}`);
    });
    render(CollectPage);

    await fireEvent.click(screen.getByRole('radio', { name: 'PseudoRNG · 1' }));

    await waitFor(() => {
      expect(screen.getByRole('note').textContent).toMatch(
        /Refresh sources and select again/,
      );
      expect(
        screen.getByRole('radio', { name: 'BitBabbler · White · 1' }),
      ).toHaveProperty('checked', true);
    });
  });
});

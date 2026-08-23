import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';
import { MOCK_SCENARIOS } from '../state/mock-scenarios';
import {
  applyDevScenario,
  getAppState,
  refreshSources,
  safeErrorMessage,
  selectSource,
} from './client';

function setTauri(enabled: boolean): void {
  const host = globalThis as { isTauri?: boolean };
  if (enabled) {
    host.isTauri = true;
  } else {
    delete host.isTauri;
  }
}

describe('ipc client', () => {
  afterEach(() => {
    setTauri(false);
    clearMocks();
  });

  it('returns the idle mock snapshot when Tauri is absent', async () => {
    const snapshot = await getAppState();
    expect(snapshot.collection.state).toBe('idle');
    expect(snapshot.collection.sessionId).toBeNull();
    const dump = JSON.stringify(snapshot);
    expect(dump).not.toMatch(/COM\d/i);
    expect(dump).not.toMatch(/[A-Za-z]:\\/);
    expect(dump).not.toMatch(/\/dev\//);
  });

  it('invokes get_app_state inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd) => {
      expect(cmd).toBe('get_app_state');
      return MOCK_SCENARIOS.idle;
    });
    await expect(getAppState()).resolves.toEqual(MOCK_SCENARIOS.idle);
  });

  it('applies a development scenario without Tauri', async () => {
    const snapshot = await applyDevScenario('ready');
    expect(snapshot.collection.state).toBe('ready');
    expect(snapshot.collection.selectedToken).toBe('mock-bitb-1');
  });

  it('reconciles development scenarios from the backend inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd, payload) => {
      expect(cmd).toBe('apply_dev_scenario');
      expect(payload).toEqual({ scenarioId: 'collecting' });
      return MOCK_SCENARIOS.collecting;
    });
    const snapshot = await applyDevScenario('collecting');
    expect(snapshot.collection.state).toBe('collecting');
  });

  it('invokes refresh_sources inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd) => {
      expect(cmd).toBe('refresh_sources');
      return MOCK_SCENARIOS.idle;
    });
    await expect(refreshSources()).resolves.toEqual(MOCK_SCENARIOS.idle);
  });

  it('invokes select_source inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd, payload) => {
      expect(cmd).toBe('select_source');
      expect(payload).toEqual({ token: 'mock-bitb-1' });
      return MOCK_SCENARIOS.ready;
    });
    await expect(selectSource('mock-bitb-1')).resolves.toEqual(
      MOCK_SCENARIOS.ready,
    );
  });

  it('uses only structured safe IPC errors in the UI', () => {
    expect(
      safeErrorMessage({
        code: 'expired_selection',
        message: 'That source is no longer valid.',
        recovery: 'Refresh sources and select again.',
      }),
    ).toBe('That source is no longer valid. Refresh sources and select again.');
    expect(safeErrorMessage(new Error('COM3 failed'))).toBe(
      'The operation failed unexpectedly.',
    );
    expect(
      safeErrorMessage({ code: 'raw_error', message: 'COM3 failed' }),
    ).toBe('The operation failed unexpectedly.');
  });
});

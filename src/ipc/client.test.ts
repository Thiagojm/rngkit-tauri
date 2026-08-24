import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';
import { MOCK_SCENARIOS } from '../state/mock-scenarios';
import {
  applyDevScenario,
  chooseOutputFolder,
  chooseCsvInputs,
  chooseReportInput,
  createDerived,
  generateDerived,
  generateReport,
  getAppState,
  openDerivedFolder,
  openReport,
  openReportFolder,
  openSessionFolder,
  replaceReport,
  refreshSources,
  safeErrorMessage,
  selectSource,
  setSampleBits,
  setTheme,
  startCollection,
  stopAndExit,
  stopCollection,
  copyDiagnostics,
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

  it('invokes session-draft commands inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd, payload) => {
      if (cmd === 'set_sample_bits') {
        expect(payload).toEqual({ bits: 16 });
        return MOCK_SCENARIOS.ready;
      }
      if (cmd === 'set_theme') {
        expect(payload).toEqual({ theme: 'dark' });
        return { ...MOCK_SCENARIOS.idle, theme: 'dark' };
      }
      if (cmd === 'choose_output_folder') {
        return MOCK_SCENARIOS.ready;
      }
      throw new Error(`unexpected command ${cmd}`);
    });
    await expect(setSampleBits(16)).resolves.toEqual(MOCK_SCENARIOS.ready);
    await expect(setTheme('dark')).resolves.toMatchObject({ theme: 'dark' });
    await expect(chooseOutputFolder()).resolves.toEqual(MOCK_SCENARIOS.ready);
  });

  it('invokes collection commands inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd) => {
      if (cmd === 'start_collection') {
        return MOCK_SCENARIOS.collecting;
      }
      if (cmd === 'stop_collection') {
        return MOCK_SCENARIOS.stopping;
      }
      if (cmd === 'open_session_folder') {
        return MOCK_SCENARIOS.completed;
      }
      throw new Error(`unexpected command ${cmd}`);
    });
    await expect(startCollection(() => undefined)).resolves.toEqual(
      MOCK_SCENARIOS.collecting,
    );
    await expect(stopCollection()).resolves.toEqual(MOCK_SCENARIOS.stopping);
    await expect(openSessionFolder()).resolves.toEqual(
      MOCK_SCENARIOS.completed,
    );
  });

  it('invokes copy_diagnostics and stop_and_exit inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd) => {
      if (cmd === 'copy_diagnostics') {
        return 'RngKit 0.1.0\nlibrary test\n';
      }
      if (cmd === 'stop_and_exit') {
        return MOCK_SCENARIOS.stopping;
      }
      throw new Error(`unexpected command ${cmd}`);
    });
    await expect(copyDiagnostics()).resolves.toContain('RngKit');
    await expect(stopAndExit()).resolves.toEqual(MOCK_SCENARIOS.stopping);
  });

  it('invokes native report commands inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd) => {
      if (cmd === 'choose_report_input') {
        return MOCK_SCENARIOS.reportsPreview;
      }
      if (cmd === 'generate_report' || cmd === 'replace_report') {
        return MOCK_SCENARIOS.reportsConflict;
      }
      if (cmd === 'open_report' || cmd === 'open_report_folder') {
        return MOCK_SCENARIOS.reportsConflict;
      }
      throw new Error(`unexpected command ${cmd}`);
    });
    await expect(chooseReportInput()).resolves.toEqual(
      MOCK_SCENARIOS.reportsPreview,
    );
    await expect(chooseReportInput(true)).resolves.toEqual(
      MOCK_SCENARIOS.reportsPreview,
    );
    await expect(generateReport()).resolves.toEqual(
      MOCK_SCENARIOS.reportsConflict,
    );
    await expect(replaceReport()).resolves.toEqual(
      MOCK_SCENARIOS.reportsConflict,
    );
    await expect(openReport()).resolves.toEqual(MOCK_SCENARIOS.reportsConflict);
    await expect(openReportFolder()).resolves.toEqual(
      MOCK_SCENARIOS.reportsConflict,
    );
  });

  it('invokes combine commands inside Tauri', async () => {
    setTauri(true);
    mockIPC((cmd) => {
      if (
        cmd === 'choose_csv_inputs' ||
        cmd === 'create_derived' ||
        cmd === 'generate_derived' ||
        cmd === 'open_derived_folder'
      ) {
        return MOCK_SCENARIOS.combineCompatible;
      }
      throw new Error(`unexpected command ${cmd}`);
    });
    await expect(chooseCsvInputs()).resolves.toEqual(
      MOCK_SCENARIOS.combineCompatible,
    );
    await expect(createDerived()).resolves.toEqual(
      MOCK_SCENARIOS.combineCompatible,
    );
    await expect(generateDerived()).resolves.toEqual(
      MOCK_SCENARIOS.combineCompatible,
    );
    await expect(openDerivedFolder()).resolves.toEqual(
      MOCK_SCENARIOS.combineCompatible,
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

import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import {
  DEFAULT_SCENARIO,
  MOCK_SCENARIOS,
  browserDiscoverySnapshot,
  type ScenarioId,
} from '../state/mock-scenarios';
import {
  ERROR_CODES,
  type AppSnapshot,
  type ClosePromptMode,
  type CollectionEvent,
  type SafeErrorDto,
  type ThemePreference,
} from './types';

const FALLBACK_ERROR_MESSAGE = 'The operation failed unexpectedly.';
const SAFE_ERROR_CODES = new Set<string>(ERROR_CODES);

function isSafeErrorDto(error: unknown): error is SafeErrorDto {
  if (typeof error !== 'object' || error === null) {
    return false;
  }
  const candidate = error as Partial<SafeErrorDto>;
  return (
    typeof candidate.code === 'string' &&
    SAFE_ERROR_CODES.has(candidate.code) &&
    typeof candidate.message === 'string'
  );
}

export function safeErrorMessage(error: unknown): string {
  if (!isSafeErrorDto(error)) {
    return FALLBACK_ERROR_MESSAGE;
  }
  return error.recovery ? `${error.message} ${error.recovery}` : error.message;
}

export async function getAppState(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('get_app_state');
  }
  return structuredClone(MOCK_SCENARIOS[DEFAULT_SCENARIO]);
}

export async function refreshSources(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('refresh_sources');
  }
  if (import.meta.env.DEV) {
    return browserDiscoverySnapshot();
  }
  return getAppState();
}

export async function selectSource(token: string): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('select_source', { token });
  }
  return getAppState();
}

export async function setSampleBits(bits: number): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('set_sample_bits', { bits });
  }
  return getAppState();
}

export async function setIntervalSeconds(
  seconds: number,
): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('set_interval_seconds', { seconds });
  }
  return getAppState();
}

export async function setFold(fold: number): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('set_fold', { fold });
  }
  return getAppState();
}

export async function setTheme(theme: ThemePreference): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('set_theme', { theme });
  }
  return getAppState();
}

export async function chooseOutputFolder(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('choose_output_folder');
  }
  if (import.meta.env.DEV) {
    const snapshot = await getAppState();
    return {
      ...snapshot,
      collection: {
        ...snapshot.collection,
        outputRootLabel: snapshot.collection.outputRootLabel ?? 'Chosen folder',
      },
    };
  }
  return getAppState();
}

export async function startCollection(
  onEvent: (event: CollectionEvent) => void,
): Promise<AppSnapshot> {
  if (isTauri()) {
    const channel = new Channel<CollectionEvent>(onEvent);
    return invoke<AppSnapshot>('start_collection', { onEvent: channel });
  }
  const snapshot = await getAppState();
  if (snapshot.collection.state !== 'ready') {
    return snapshot;
  }
  return {
    ...snapshot,
    collection: {
      ...snapshot.collection,
      state: 'collecting',
      statusLabel: 'Collecting',
      sessionId: 's1',
      lastEventSequence: 0,
      errorCode: null,
      errorMessage: null,
      errorRecovery: null,
    },
  };
}

export async function stopCollection(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('stop_collection');
  }
  const snapshot = await getAppState();
  if (
    snapshot.collection.state !== 'collecting' &&
    snapshot.collection.state !== 'stopping'
  ) {
    return snapshot;
  }
  return {
    ...snapshot,
    collection: {
      ...snapshot.collection,
      state: 'completed',
      statusLabel: 'Completed',
    },
  };
}

export async function startAnotherSession(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('start_another_session');
  }
  const snapshot = await getAppState();
  if (
    snapshot.collection.state !== 'completed' &&
    snapshot.collection.state !== 'failed'
  ) {
    return snapshot;
  }
  return {
    ...snapshot,
    collection: {
      ...snapshot.collection,
      state: snapshot.collection.outputRootLabel ? 'ready' : 'idle',
      statusLabel: snapshot.collection.outputRootLabel ? 'Ready' : 'Idle',
      sampleCount: 0,
      elapsedLabel: '00:00:00',
      onesProportionLabel: '—',
      cumulativeZLabel: '—',
      overrunCount: 0,
      sessionStem: null,
      sessionId: null,
      lastEventSequence: 0,
      errorCode: null,
      errorMessage: null,
      errorRecovery: null,
    },
  };
}

export async function openSessionFolder(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('open_session_folder');
  }
  return getAppState();
}

export async function chooseReportInput(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('choose_report_input');
  }
  return getAppState();
}

export async function generateReport(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('generate_report');
  }
  return getAppState();
}

export async function replaceReport(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('replace_report');
  }
  return getAppState();
}

export async function openReport(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('open_report');
  }
  return getAppState();
}

export async function openReportFolder(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('open_report_folder');
  }
  return getAppState();
}

export async function chooseCsvInputs(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('choose_csv_inputs');
  }
  return getAppState();
}

export async function removeCombineInput(
  inputId: string,
): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('remove_combine_input', { inputId });
  }
  return getAppState();
}

export async function clearCombineInputs(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('clear_combine_inputs');
  }
  return getAppState();
}

export async function createDerived(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('create_derived');
  }
  return getAppState();
}

export async function generateDerived(replace = false): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('generate_derived', { replace });
  }
  return getAppState();
}

export async function openDerivedFolder(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('open_derived_folder');
  }
  return getAppState();
}

function formatMockDiagnostics(snapshot: AppSnapshot): string {
  if (snapshot.diagnostics.length === 0) {
    return 'RngKit diagnostics\nNo diagnostic records.';
  }
  const first = snapshot.diagnostics[0];
  const lines = [
    `RngKit ${first.appVersion}`,
    `library ${first.libraryRevision}`,
    '',
  ];
  for (const record of snapshot.diagnostics) {
    lines.push(`${record.operationId} ${record.code}`, record.detail, '');
  }
  return lines.join('\n');
}

export async function copyDiagnostics(snapshot?: AppSnapshot): Promise<string> {
  if (isTauri()) {
    return invoke<string>('copy_diagnostics');
  }
  return formatMockDiagnostics(snapshot ?? (await getAppState()));
}

export async function stopAndExit(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('stop_and_exit');
  }
  return stopCollection();
}

export async function listenCloseRequested(
  onPrompt: (mode: ClosePromptMode) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return () => undefined;
  }
  const { listen } = await import('@tauri-apps/api/event');
  return listen<{ mode: ClosePromptMode }>(
    'rngkit-close-requested',
    (event) => {
      if (
        event.payload.mode === 'confirm' ||
        event.payload.mode === 'finalizing'
      ) {
        onPrompt(event.payload.mode);
      }
    },
  );
}

export async function applyDevScenario(id: ScenarioId): Promise<AppSnapshot> {
  if (import.meta.env.DEV && isTauri()) {
    return invoke<AppSnapshot>('apply_dev_scenario', { scenarioId: id });
  }
  if (import.meta.env.DEV) {
    return structuredClone(MOCK_SCENARIOS[id]);
  }
  return getAppState();
}

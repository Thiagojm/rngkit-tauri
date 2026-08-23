import { invoke, isTauri } from '@tauri-apps/api/core';
import {
  DEFAULT_SCENARIO,
  MOCK_SCENARIOS,
  browserDiscoverySnapshot,
  type ScenarioId,
} from '../state/mock-scenarios';
import { ERROR_CODES, type AppSnapshot, type SafeErrorDto } from './types';

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

export async function applyDevScenario(id: ScenarioId): Promise<AppSnapshot> {
  if (import.meta.env.DEV && isTauri()) {
    return invoke<AppSnapshot>('apply_dev_scenario', { scenarioId: id });
  }
  if (import.meta.env.DEV) {
    return structuredClone(MOCK_SCENARIOS[id]);
  }
  return getAppState();
}

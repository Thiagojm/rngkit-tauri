import { invoke, isTauri } from '@tauri-apps/api/core';
import {
  DEFAULT_SCENARIO,
  MOCK_SCENARIOS,
  type ScenarioId,
} from '../state/mock-scenarios';
import type { AppSnapshot } from './types';

export async function getAppState(): Promise<AppSnapshot> {
  if (isTauri()) {
    return invoke<AppSnapshot>('get_app_state');
  }
  return structuredClone(MOCK_SCENARIOS[DEFAULT_SCENARIO]);
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

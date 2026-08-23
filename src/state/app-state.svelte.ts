import { isTauri } from '@tauri-apps/api/core';
import { applyDevScenario, getAppState } from '../ipc/client';
import { deriveControls } from './controls';
import {
  DEFAULT_SCENARIO,
  MOCK_SCENARIOS,
  type ScenarioId,
} from './mock-scenarios';
import type { AppSnapshot, Destination, ThemePreference } from './types';

function cloneSnapshot(id: ScenarioId): AppSnapshot {
  return structuredClone(MOCK_SCENARIOS[id]);
}

class AppViewState {
  destination = $state<Destination>('collect');
  theme = $state<ThemePreference>('system');
  scenarioId = $state<ScenarioId>(DEFAULT_SCENARIO);
  selectedToken = $state<string | null>(
    MOCK_SCENARIOS[DEFAULT_SCENARIO].collection.selectedToken,
  );
  replaceDialogOpen = $state(false);
  backendSnapshot = $state<AppSnapshot>(cloneSnapshot(DEFAULT_SCENARIO));
  loadGeneration = 0;

  snapshot = $derived({
    ...this.backendSnapshot,
    collection: {
      ...this.backendSnapshot.collection,
      selectedToken: this.selectedToken,
    },
  });
  controls = $derived(deriveControls(this.snapshot));

  reconcile(snapshot: AppSnapshot): void {
    this.backendSnapshot = snapshot;
    this.selectedToken = snapshot.collection.selectedToken;
  }

  async hydrate(): Promise<void> {
    const generation = ++this.loadGeneration;
    const snapshot = await getAppState();
    if (generation !== this.loadGeneration) {
      return;
    }
    this.reconcile(snapshot);
  }

  applyScenario(id: ScenarioId): void {
    const generation = ++this.loadGeneration;
    this.scenarioId = id;
    this.replaceDialogOpen = false;
    this.reconcile(cloneSnapshot(id));
    if (import.meta.env.DEV && isTauri()) {
      void applyDevScenario(id).then((snapshot) => {
        if (generation === this.loadGeneration && this.scenarioId === id) {
          this.reconcile(snapshot);
        }
      });
    }
  }

  selectSource(token: string): void {
    if (
      this.snapshot.collection.candidates.some(
        (candidate) => candidate.token === token,
      )
    ) {
      this.selectedToken = token;
    }
  }

  reset(): void {
    this.loadGeneration += 1;
    this.destination = 'collect';
    this.theme = 'system';
    this.scenarioId = DEFAULT_SCENARIO;
    this.replaceDialogOpen = false;
    this.reconcile(cloneSnapshot(DEFAULT_SCENARIO));
  }
}

export const appState = new AppViewState();

export function resetAppState(): void {
  appState.reset();
}

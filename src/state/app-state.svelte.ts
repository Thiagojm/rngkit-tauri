import { isTauri } from '@tauri-apps/api/core';
import {
  applyDevScenario,
  getAppState,
  refreshSources,
  safeErrorMessage,
  selectSource,
} from '../ipc/client';
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

  async reconcileCommandFailure(
    generation: number,
    fallback: AppSnapshot,
    error: unknown,
  ): Promise<void> {
    if (generation !== this.loadGeneration) {
      return;
    }
    let snapshot = fallback;
    try {
      snapshot = await getAppState();
    } catch {
      // Keep the last usable snapshot when state reconciliation also fails.
    }
    if (generation !== this.loadGeneration) {
      return;
    }
    this.reconcile({
      ...snapshot,
      collection: {
        ...snapshot.collection,
        familyWarning: safeErrorMessage(error),
      },
    });
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
      !this.snapshot.collection.candidates.some(
        (candidate) => candidate.token === token,
      )
    ) {
      return;
    }
    if (isTauri()) {
      const generation = ++this.loadGeneration;
      const fallback = $state.snapshot(this.snapshot);
      this.selectedToken = token;
      void selectSource(token)
        .then((snapshot) => {
          if (generation === this.loadGeneration) {
            this.reconcile(snapshot);
          }
        })
        .catch((error: unknown) =>
          this.reconcileCommandFailure(generation, fallback, error),
        );
      return;
    }
    this.selectedToken = token;
  }

  async refreshSources(): Promise<void> {
    const generation = ++this.loadGeneration;
    const fallback = $state.snapshot(this.snapshot);
    this.backendSnapshot = {
      ...this.backendSnapshot,
      collection: {
        ...this.backendSnapshot.collection,
        state: 'discovering',
        statusLabel: 'Discovering sources',
        candidates: [],
        selectedToken: null,
        familyWarning: null,
      },
    };
    this.selectedToken = null;
    try {
      const snapshot = await refreshSources();
      if (generation !== this.loadGeneration) {
        return;
      }
      this.reconcile(snapshot);
    } catch (error) {
      await this.reconcileCommandFailure(generation, fallback, error);
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

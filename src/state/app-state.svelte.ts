import { deriveControls } from './controls';
import {
  DEFAULT_SCENARIO,
  MOCK_SCENARIOS,
  type ScenarioId,
} from './mock-scenarios';
import type { Destination, ThemePreference } from './types';

class AppViewState {
  destination = $state<Destination>('collect');
  theme = $state<ThemePreference>('system');
  scenarioId = $state<ScenarioId>(DEFAULT_SCENARIO);
  selectedToken = $state<string | null>(
    MOCK_SCENARIOS[DEFAULT_SCENARIO].collection.selectedToken,
  );
  replaceDialogOpen = $state(false);

  snapshot = $derived({
    ...MOCK_SCENARIOS[this.scenarioId],
    collection: {
      ...MOCK_SCENARIOS[this.scenarioId].collection,
      selectedToken: this.selectedToken,
    },
  });
  controls = $derived(deriveControls(this.snapshot));

  applyScenario(id: ScenarioId): void {
    this.scenarioId = id;
    this.selectedToken = MOCK_SCENARIOS[id].collection.selectedToken;
    this.replaceDialogOpen = false;
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
    this.destination = 'collect';
    this.theme = 'system';
    this.scenarioId = DEFAULT_SCENARIO;
    this.selectedToken =
      MOCK_SCENARIOS[DEFAULT_SCENARIO].collection.selectedToken;
    this.replaceDialogOpen = false;
  }
}

export const appState = new AppViewState();

export function resetAppState(): void {
  appState.reset();
}

<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import {
    SCENARIO_IDS,
    SCENARIO_LABELS,
    type ScenarioId,
  } from '../../state/mock-scenarios';
  import { applyTheme } from '../../state/theme';
  import { statusTone } from '../../state/controls';
  import CombinePage from '../../pages/CombinePage.svelte';
  import CollectPage from '../../pages/CollectPage.svelte';
  import HelpPage from '../../pages/HelpPage.svelte';
  import ReportsPage from '../../pages/ReportsPage.svelte';
  import Navigation from './Navigation.svelte';
  import ThemeControl from './ThemeControl.svelte';

  $effect(() => {
    applyTheme(appState.theme);
  });

  function onScenarioChange(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if ((SCENARIO_IDS as readonly string[]).includes(value)) {
      appState.applyScenario(value as ScenarioId);
    }
  }
</script>

<a
  class="sr-only focus:not-sr-only focus:absolute focus:z-20 focus:m-2 focus:rounded-md focus:bg-surface focus:px-3 focus:py-2"
  href="#main-content">{copy.skipToMain}</a
>
<div class="flex min-h-svh flex-col bg-surface text-text">
  <header
    class="flex flex-wrap items-center gap-3 border-b border-border px-4 py-3"
  >
    <p class="text-lg font-semibold tracking-tight">{copy.product}</p>
    <p
      class="text-sm {statusTone(appState.snapshot.collection.state)}"
      aria-live="polite"
    >
      {copy.status}: {appState.snapshot.collection.statusLabel}
    </p>
    <div class="ml-auto flex flex-wrap items-center gap-3">
      <ThemeControl />
      {#if import.meta.env.DEV}
        <label class="flex items-center gap-2 text-sm">
          Development scenario
          <select
            class="rounded-md border border-border bg-surface px-2 py-1"
            data-testid="dev-scenario-switch"
            value={appState.scenarioId}
            onchange={onScenarioChange}
          >
            {#each SCENARIO_IDS as id (id)}
              <option value={id}>{SCENARIO_LABELS[id]}</option>
            {/each}
          </select>
        </label>
      {/if}
    </div>
  </header>
  <div class="flex min-h-0 min-w-0 flex-1">
    <Navigation />
    <main
      id="main-content"
      class="min-w-0 flex-1 overflow-auto px-4 py-4 narrow:px-6"
      tabindex="-1"
    >
      {#if appState.destination === 'collect'}
        <CollectPage />
      {:else if appState.destination === 'reports'}
        <ReportsPage />
      {:else if appState.destination === 'combine'}
        <CombinePage />
      {:else}
        <HelpPage />
      {/if}
    </main>
  </div>
</div>

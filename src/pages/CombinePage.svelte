<script lang="ts">
  import CombineActions from '../components/combine/CombineActions.svelte';
  import CompatibilitySummary from '../components/combine/CompatibilitySummary.svelte';
  import InputTable from '../components/combine/InputTable.svelte';
  import Button from '../components/ui/Button.svelte';
  import { copy } from '../copy';
  import { appState } from '../state/app-state.svelte';

  const controls = $derived(appState.controls);
</script>

<div class="flex min-w-0 flex-col gap-4">
  <div class="flex flex-wrap items-start justify-between gap-2">
    <h1 class="text-2xl font-semibold">{copy.destinations.combine}</h1>
    <Button
      variant="ghost"
      disabled={!controls.openCombineWorkingFolder.enabled}
      disabledReason={controls.openCombineWorkingFolder.reason}
      onclick={() => appState.openCombineWorkingFolder()}
      >{copy.openWorkingFolder}</Button
    >
  </div>
  <p class="text-text-muted">
    Add compatible current or legacy CSV files from one or more folders and
    create a provenance-bearing derived bundle without modifying the inputs.
  </p>
  <CombineActions />
  <InputTable />
  <CompatibilitySummary />
</div>

<script lang="ts">
  import Button from '../components/ui/Button.svelte';
  import { copy, FOLD_OPTIONS } from '../copy';
  import { appState } from '../state/app-state.svelte';

  const combine = $derived(appState.snapshot.combine);
  const controls = $derived(appState.controls);

  function foldLabel(fold: number | null): string {
    if (fold === null) {
      return '—';
    }
    return (
      FOLD_OPTIONS.find((option) => option.value === fold)?.label ??
      String(fold)
    );
  }
</script>

<div class="flex min-w-0 flex-col gap-4">
  <h1 class="text-2xl font-semibold">{copy.destinations.combine}</h1>
  <p class="text-text-muted">
    Preview compatible RngKitPSG v3 CSV files and create a provenance-bearing
    derived bundle without modifying the inputs.
  </p>
  <Button
    disabled={!controls.combine.enabled}
    disabledReason={controls.combine.reason}>{copy.chooseCsvInputs}</Button
  >
  {#if combine.inputs.length === 0}
    <p class="text-sm text-text-muted">{copy.noCombineInputs}</p>
  {:else}
    <div class="min-w-0 overflow-x-auto">
      <table class="w-full min-w-[36rem] border-collapse text-left text-sm">
        <caption class="sr-only"
          >Legacy CSV inputs in derived chronological order</caption
        >
        <thead>
          <tr class="border-b border-border">
            <th class="px-2 py-2 font-medium">Basename</th>
            <th class="px-2 py-2 font-medium">Source</th>
            <th class="px-2 py-2 font-medium">Bits</th>
            <th class="px-2 py-2 font-medium">Interval</th>
            <th class="px-2 py-2 font-medium">{copy.fold.label}</th>
            <th class="px-2 py-2 font-medium">First</th>
            <th class="px-2 py-2 font-medium">Last</th>
            <th class="px-2 py-2 font-medium">Rows</th>
            <th class="px-2 py-2 font-medium">State</th>
          </tr>
        </thead>
        <tbody>
          {#each combine.inputs as row (row.basename)}
            <tr class="border-b border-border">
              <td class="px-2 py-2 font-mono">{row.basename}</td>
              <td class="px-2 py-2">{row.source}</td>
              <td class="px-2 py-2">{row.sampleBits}</td>
              <td class="px-2 py-2">{row.intervalSeconds}</td>
              <td class="px-2 py-2">{foldLabel(row.fold)}</td>
              <td class="px-2 py-2 font-mono">{row.firstTimestamp}</td>
              <td class="px-2 py-2 font-mono">{row.lastTimestamp}</td>
              <td class="px-2 py-2">{row.rows}</td>
              <td class="px-2 py-2">
                {#if row.valid}
                  Valid
                {:else}
                  {row.error}
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    {#if combine.incompatibility}
      <p class="text-sm text-status-failed" role="status">
        {combine.incompatibility}
      </p>
    {/if}
  {/if}
  <div class="flex flex-wrap gap-2">
    <Button
      variant="primary"
      disabled={!controls.createDerived.enabled}
      disabledReason={controls.createDerived.reason}
      >{copy.createDerived}</Button
    >
    <Button
      disabled={!combine.result}
      disabledReason="Create a derived bundle before generating XLSX."
      >{copy.generateXlsx}</Button
    >
    <Button
      disabled={!combine.result}
      disabledReason="Create a derived bundle before opening its folder."
      >{copy.openFolder}</Button
    >
  </div>
  {#if combine.result}
    <dl
      class="grid grid-cols-1 gap-3 rounded-md border border-border p-4 sm:grid-cols-3"
    >
      <div>
        <dt class="text-sm text-text-muted">Bundle stem</dt>
        <dd class="font-mono text-sm">{combine.result.stem}</dd>
      </div>
      <div>
        <dt class="text-sm text-text-muted">Inputs</dt>
        <dd>{combine.result.inputCount}</dd>
      </div>
      <div>
        <dt class="text-sm text-text-muted">Total rows</dt>
        <dd>{combine.result.totalRows}</dd>
      </div>
    </dl>
  {/if}
</div>

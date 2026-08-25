<script lang="ts">
  import { copy, FOLD_OPTIONS } from '../../copy';
  import { appState } from '../../state/app-state.svelte';

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

  function formatLabel(format: string): string {
    if (format === 'current_csv') return 'Current CSV';
    if (format === 'legacy_v3_csv') return 'Legacy v3 CSV';
    return 'Unknown';
  }
</script>

{#if combine.inputs.length === 0}
  <p class="text-sm text-text-muted">{copy.noCombineInputs}</p>
{:else}
  <div class="min-w-0 max-w-full overflow-x-auto">
    <table class="w-full min-w-[36rem] border-collapse text-left text-sm">
      <caption class="sr-only"
        >CSV inputs in derived chronological order</caption
      >
      <thead>
        <tr class="border-b border-border">
          <th class="px-2 py-2 font-medium">#</th>
          <th class="px-2 py-2 font-medium">Basename</th>
          <th class="px-2 py-2 font-medium">Format</th>
          <th class="px-2 py-2 font-medium">Source</th>
          <th class="px-2 py-2 font-medium">Bits</th>
          <th class="px-2 py-2 font-medium">Interval</th>
          <th class="px-2 py-2 font-medium">{copy.fold.label}</th>
          <th class="px-2 py-2 font-medium">First</th>
          <th class="px-2 py-2 font-medium">Last</th>
          <th class="px-2 py-2 font-medium">Rows</th>
          <th class="px-2 py-2 font-medium">State</th>
          <th class="px-2 py-2 font-medium"
            ><span class="sr-only">Action</span></th
          >
        </tr>
      </thead>
      <tbody>
        {#each combine.inputs as row (row.inputId)}
          <tr class="border-b border-border">
            <td class="px-2 py-2">{row.ordinal}</td>
            <td class="px-2 py-2 font-mono">{row.basename}</td>
            <td class="px-2 py-2">{formatLabel(row.format)}</td>
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
            <td class="px-2 py-2">
              <button
                class="text-sm text-status-failed underline disabled:text-text-muted disabled:no-underline"
                type="button"
                disabled={!controls.combine.enabled}
                aria-label={`${copy.removeCombineInput} ${row.basename} #${row.ordinal}`}
                onclick={() => appState.removeCombineInput(row.inputId)}
                >{copy.removeCombineInput}</button
              >
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<script lang="ts">
  import { copy, FOLD_OPTIONS } from '../../copy';
  import { appState } from '../../state/app-state.svelte';

  const combine = $derived(appState.snapshot.combine);

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
{/if}

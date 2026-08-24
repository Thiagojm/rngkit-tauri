<script lang="ts">
  import { appState } from '../../state/app-state.svelte';

  const combine = $derived(appState.snapshot.combine);
</script>

{#if combine.incompatibility}
  <p class="text-sm text-status-failed" role="status">
    {combine.incompatibility}
  </p>
{/if}
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

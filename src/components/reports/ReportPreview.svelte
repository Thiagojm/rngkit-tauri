<script lang="ts">
  import { copy, FOLD_OPTIONS } from '../../copy';
  import { appState } from '../../state/app-state.svelte';

  const preview = $derived(appState.snapshot.reports.preview);

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

{#if preview}
  <dl
    class="grid grid-cols-1 gap-3 rounded-md border border-border p-4 sm:grid-cols-2"
  >
    <div>
      <dt class="text-sm text-text-muted">Kind</dt>
      <dd>{preview.kindLabel}</dd>
    </div>
    <div>
      <dt class="text-sm text-text-muted">Origin</dt>
      <dd>{preview.origin}</dd>
    </div>
    <div>
      <dt class="text-sm text-text-muted">Source</dt>
      <dd>{preview.source}</dd>
    </div>
    <div>
      <dt class="text-sm text-text-muted">{copy.sampleBits}</dt>
      <dd>{preview.sampleBits}</dd>
    </div>
    <div>
      <dt class="text-sm text-text-muted">{copy.sampleInterval}</dt>
      <dd>{preview.intervalSeconds}</dd>
    </div>
    <div>
      <dt class="text-sm text-text-muted">{copy.fold.label}</dt>
      <dd>{foldLabel(preview.fold)}</dd>
    </div>
    <div>
      <dt class="text-sm text-text-muted">Status</dt>
      <dd>{preview.status}</dd>
    </div>
    <div>
      <dt class="text-sm text-text-muted">Rows</dt>
      <dd>{preview.rowCount}</dd>
    </div>
  </dl>
  {#if preview.warning}
    <p class="text-sm text-text-muted" role="note">{preview.warning}</p>
  {/if}
  {#if preview.conflict}
    <p class="text-sm text-text-muted" role="note">
      An XLSX file already exists. Generating asks to cancel or replace it.
    </p>
  {/if}
{/if}

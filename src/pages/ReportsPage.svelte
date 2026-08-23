<script lang="ts">
  import Button from '../components/ui/Button.svelte';
  import Dialog from '../components/ui/Dialog.svelte';
  import { copy, FOLD_OPTIONS } from '../copy';
  import { appState } from '../state/app-state.svelte';

  const preview = $derived(appState.snapshot.reports.preview);
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

  function onGenerate(): void {
    if (!preview) {
      return;
    }
    if (preview.conflict) {
      appState.replaceDialogOpen = true;
    }
  }
</script>

<div class="flex max-w-3xl flex-col gap-4">
  <h1 class="text-2xl font-semibold">{copy.destinations.reports}</h1>
  <p class="text-text-muted">
    Inspect a native session, legacy v3 BIN or CSV, or a derived concatenation
    bundle, then generate a same-stem XLSX report.
  </p>
  <Button
    disabled={!controls.reports.enabled}
    disabledReason={controls.reports.reason}>{copy.chooseReportInput}</Button
  >
  {#if !preview}
    <p class="text-sm text-text-muted">{copy.noReportInput}</p>
  {:else}
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
  <div class="flex flex-wrap gap-2">
    <Button
      variant="primary"
      disabled={!controls.generateReport.enabled}
      disabledReason={controls.generateReport.reason}
      onclick={onGenerate}>{copy.generateReport}</Button
    >
    <Button
      disabled={!preview}
      disabledReason="Open the report after a report is generated."
      >{copy.openReport}</Button
    >
    <Button
      disabled={!preview}
      disabledReason="Open the containing folder after a report exists."
      >{copy.openContainingFolder}</Button
    >
  </div>
</div>

<Dialog
  title={copy.replaceTitle}
  open={appState.replaceDialogOpen}
  onClose={() => (appState.replaceDialogOpen = false)}
>
  <p>{copy.replaceBody}</p>
  {#snippet actions()}
    <Button
      variant="primary"
      onclick={() => (appState.replaceDialogOpen = false)}>{copy.cancel}</Button
    >
    <Button onclick={() => (appState.replaceDialogOpen = false)}
      >{copy.replace}</Button
    >
  {/snippet}
</Dialog>

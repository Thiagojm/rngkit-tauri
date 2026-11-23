<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';
  import Dialog from '../ui/Dialog.svelte';

  const combine = $derived(appState.snapshot.combine);
  const controls = $derived(appState.controls);
</script>

{#if combine.incompatibility}
  <p class="text-sm text-status-failed" role="status">
    {combine.incompatibility}
  </p>
{/if}
<Button
  variant="primary"
  disabled={!controls.createDerived.enabled}
  disabledReason={controls.createDerived.reason}
  onclick={() => appState.createDerived()}>{copy.createDerived}</Button
>
{#if combine.result}
  <div class="flex flex-col gap-3 rounded-md border border-border p-4">
    <dl
      class="grid grid-cols-1 gap-3 sm:grid-cols-[minmax(0,2fr)_minmax(0,1fr)_minmax(0,1fr)]"
    >
      <div>
        <dt class="text-sm text-text-muted">{copy.combinedFileName}</dt>
        <dd class="break-all font-mono text-sm">{combine.result.stem}</dd>
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
    <div class="flex flex-wrap gap-2">
      <Button onclick={() => appState.generateDerived()}
        >{copy.generateXlsx}</Button
      >
      <Button onclick={() => appState.openDerivedFolder()}
        >{copy.openCombinedFolder}</Button
      >
    </div>
  </div>
{/if}

<Dialog
  title={copy.replaceTitle}
  open={appState.replaceDialogOpen}
  onClose={() => appState.cancelReplaceReport()}
>
  <p>{copy.replaceBody}</p>
  {#snippet actions()}
    <Button variant="primary" onclick={() => appState.cancelReplaceReport()}
      >{copy.cancel}</Button
    >
    <Button onclick={() => appState.confirmReplaceDerived()}
      >{copy.replace}</Button
    >
  {/snippet}
</Dialog>

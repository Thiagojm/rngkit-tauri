<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from './Button.svelte';

  const collection = $derived(appState.snapshot.collection);
  const diagnostics = $derived(appState.snapshot.diagnostics);
  const visible = $derived(
    Boolean(collection.errorMessage) || diagnostics.length > 0,
  );

  async function onCopy(): Promise<void> {
    await appState.copyDiagnostics();
  }
</script>

{#if visible}
  <section
    class="flex flex-col gap-3 rounded-md border border-border bg-surface-muted px-4 py-3"
    aria-labelledby="error-panel-title"
    data-testid="error-panel"
  >
    <h2 id="error-panel-title" class="font-medium">
      {collection.statusLabel}
    </h2>
    {#if collection.errorMessage}
      <p class="text-sm">{collection.errorMessage}</p>
    {/if}
    {#if collection.errorRecovery}
      <p class="text-sm text-text-muted">{collection.errorRecovery}</p>
    {/if}
    <div class="flex flex-wrap items-center gap-2">
      <Button
        disabled={diagnostics.length === 0}
        disabledReason={copy.errors.empty}
        onclick={onCopy}>{copy.errors.copyDiagnostics}</Button
      >
      {#if appState.diagnosticsCopied}
        <p class="text-sm text-text-muted" role="status">
          {copy.errors.copied}
        </p>
      {/if}
    </div>
  </section>
{/if}

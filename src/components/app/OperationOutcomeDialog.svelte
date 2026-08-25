<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import type { OutcomeActionId, OutcomeNotice } from '../../state/types';
  import Button from '../ui/Button.svelte';
  import Dialog from '../ui/Dialog.svelte';

  const outcome = $derived(appState.outcome);
  let resolving = $state(false);

  const actionLabels: Record<OutcomeActionId, string> = {
    openSessionFolder: copy.outcome.openSessionFolder,
    openReport: copy.outcome.openReport,
    openReportFolder: copy.outcome.openReportFolder,
    openDerivedFolder: copy.outcome.openDerivedFolder,
    openCollectionWorkingFolder: copy.outcome.openCollectionWorkingFolder,
    openReportWorkingFolder: copy.outcome.openReportWorkingFolder,
    openCombineWorkingFolder: copy.outcome.openCombineWorkingFolder,
  };

  const severityClass = $derived(
    outcome?.severity === 'error'
      ? 'text-status-failed'
      : outcome?.severity === 'warning'
        ? 'text-status-collecting'
        : 'text-status-ready',
  );
  const blocked = $derived(
    appState.replaceDialogOpen || appState.closePrompt !== 'none',
  );

  function close(): void {
    if (!outcome || resolving) {
      return;
    }
    resolving = true;
    void appState.resolveOutcome(outcome.id).finally(() => {
      resolving = false;
    });
  }

  function runAction(action: OutcomeActionId): void {
    if (!outcome || resolving) {
      return;
    }
    resolving = true;
    void appState.resolveOutcome(outcome.id, action).finally(() => {
      resolving = false;
    });
  }

  function isBackendOutcome(
    value: OutcomeNotice | null,
  ): value is OutcomeNotice {
    return value !== null && value.id > 0;
  }
</script>

<Dialog
  title={outcome?.title ?? ''}
  open={Boolean(outcome) && !blocked}
  onClose={close}
>
  {#if outcome}
    <div class="flex flex-col gap-4" aria-live="polite">
      <p class={severityClass}>{outcome.message}</p>
      {#if outcome.paths.length > 0}
        <ul
          class="flex max-h-48 flex-col gap-2 overflow-auto rounded-md border border-border p-3"
        >
          {#each outcome.paths as row (row.path)}
            <li class="flex flex-col gap-1">
              <span class="text-xs font-medium text-text-muted"
                >{row.label}</span
              >
              <span class="select-all break-all font-mono text-xs text-text"
                >{row.path}</span
              >
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
  {#snippet actions()}
    {#if outcome && isBackendOutcome(outcome)}
      {#each outcome.actions as action (action)}
        <Button disabled={resolving} onclick={() => runAction(action)}
          >{actionLabels[action]}</Button
        >
      {/each}
    {/if}
    <Button variant="primary" disabled={resolving} onclick={close}
      >{copy.outcome.dismiss}</Button
    >
  {/snippet}
</Dialog>

<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';

  const preview = $derived(appState.snapshot.reports.preview);
  const controls = $derived(appState.controls);
</script>

<div class="flex flex-wrap gap-2">
  <Button
    disabled={!controls.reports.enabled}
    disabledReason={controls.reports.reason}
    onclick={() => appState.chooseReportInput()}
    >{copy.chooseReportInput}</Button
  >
  <Button
    disabled={!controls.reports.enabled}
    disabledReason={controls.reports.reason}
    onclick={() => appState.chooseReportInput(true)}
    >{copy.chooseLegacyInput}</Button
  >
</div>
{#if appState.snapshot.preferencesWarning}
  <p class="text-sm text-text-muted" role="alert">
    {appState.snapshot.preferencesWarning}
  </p>
{/if}
{#if !preview}
  <p class="text-sm text-text-muted">{copy.noReportInput}</p>
{/if}

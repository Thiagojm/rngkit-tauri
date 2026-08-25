<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';
  import Dialog from '../ui/Dialog.svelte';

  const controls = $derived(appState.controls);
</script>

<div class="flex flex-wrap gap-2">
  <Button
    variant="primary"
    disabled={!controls.generateReport.enabled}
    disabledReason={controls.generateReport.reason}
    onclick={() => appState.generateReport()}>{copy.generateReport}</Button
  >
  <Button
    disabled={!controls.openReport.enabled}
    disabledReason={controls.openReport.reason}
    onclick={() => appState.openReport()}>{copy.openReport}</Button
  >
  <Button
    disabled={!controls.openContainingFolder.enabled}
    disabledReason={controls.openContainingFolder.reason}
    onclick={() => appState.openReportFolder()}
    >{copy.openContainingFolder}</Button
  >
  <Button
    disabled={!controls.openReportWorkingFolder.enabled}
    disabledReason={controls.openReportWorkingFolder.reason}
    onclick={() => appState.openReportWorkingFolder()}
    >{copy.openWorkingFolder}</Button
  >
</div>

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
    <Button onclick={() => appState.confirmReplaceReport()}
      >{copy.replace}</Button
    >
  {/snippet}
</Dialog>

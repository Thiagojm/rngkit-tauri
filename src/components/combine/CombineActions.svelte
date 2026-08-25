<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';
  import Dialog from '../ui/Dialog.svelte';

  const combine = $derived(appState.snapshot.combine);
  const controls = $derived(appState.controls);
</script>

<div class="flex flex-wrap gap-2">
  <Button
    disabled={!controls.combine.enabled}
    disabledReason={controls.combine.reason}
    onclick={() => appState.chooseCsvInputs()}>{copy.chooseCsvInputs}</Button
  >
  <Button
    disabled={!controls.combine.enabled || combine.inputs.length === 0}
    disabledReason={controls.combine.enabled
      ? 'Select at least one CSV file before clearing the selection.'
      : controls.combine.reason}
    onclick={() => appState.clearCombineInputs()}
    >{copy.clearCombineInputs}</Button
  >
  <Button
    variant="primary"
    disabled={!controls.createDerived.enabled}
    disabledReason={controls.createDerived.reason}
    onclick={() => appState.createDerived()}>{copy.createDerived}</Button
  >
  <Button
    disabled={!combine.result}
    disabledReason="Create a derived bundle before generating XLSX."
    onclick={() => appState.generateDerived()}>{copy.generateXlsx}</Button
  >
  <Button
    disabled={!combine.result}
    disabledReason="Create a derived bundle before opening its folder."
    onclick={() => appState.openDerivedFolder()}>{copy.openFolder}</Button
  >
  <Button
    disabled={!controls.openCombineWorkingFolder.enabled}
    disabledReason={controls.openCombineWorkingFolder.reason}
    onclick={() => appState.openCombineWorkingFolder()}
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
    <Button onclick={() => appState.confirmReplaceDerived()}
      >{copy.replace}</Button
    >
  {/snippet}
</Dialog>

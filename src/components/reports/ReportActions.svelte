<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import ActionGroup from '../ui/ActionGroup.svelte';
  import Button from '../ui/Button.svelte';
  import Dialog from '../ui/Dialog.svelte';

  const controls = $derived(appState.controls);
</script>

<ActionGroup
  actions={[
    {
      label: copy.generateReport,
      variant: 'primary',
      ...controls.generateReport,
      onclick: () => appState.generateReport(),
    },
    {
      label: copy.openReport,
      ...controls.openReport,
      onclick: () => appState.openReport(),
    },
    {
      label: copy.openContainingFolder,
      ...controls.openContainingFolder,
      onclick: () => appState.openReportFolder(),
    },
    {
      label: copy.openWorkingFolder,
      ...controls.openReportWorkingFolder,
      onclick: () => appState.openReportWorkingFolder(),
    },
  ]}
/>

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

<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import ActionGroup from '../ui/ActionGroup.svelte';

  const combine = $derived(appState.snapshot.combine);
  const controls = $derived(appState.controls);
</script>

<ActionGroup
  actions={[
    {
      label: copy.chooseCsvInputs,
      ...controls.combine,
      onclick: () => appState.chooseCsvInputs(),
    },
    {
      label: copy.clearCombineInputs,
      enabled: controls.combine.enabled && combine.inputs.length > 0,
      reason: controls.combine.enabled
        ? 'Select at least one CSV file before clearing the selection.'
        : controls.combine.reason,
      onclick: () => appState.clearCombineInputs(),
    },
  ]}
/>

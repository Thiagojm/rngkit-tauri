<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';

  const controls = $derived(appState.controls);
</script>

{#if controls.showStart}
  <Button
    variant="primary"
    disabled={!controls.start.enabled}
    disabledReason={controls.start.reason}
    onclick={() => appState.startCollection()}>{copy.start}</Button
  >
{/if}
{#if controls.showStop}
  <Button
    variant="primary"
    disabled={!controls.stop.enabled}
    disabledReason={controls.stop.reason}
    onclick={() => appState.stopCollection()}>{copy.stop}</Button
  >
{/if}

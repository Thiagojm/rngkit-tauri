<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';

  let {
    onStart,
    submitting = false,
  }: { onStart: () => void; submitting?: boolean } = $props();

  const controls = $derived(appState.controls);
</script>

{#if controls.showStart}
  <Button
    variant="primary"
    disabled={!controls.start.enabled || submitting}
    disabledReason={controls.start.reason}
    onclick={onStart}>{copy.start}</Button
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

{#if controls.showTerminalActions}
  <div class="flex flex-wrap gap-2">
    <Button
      disabled={!controls.openSessionFolder.enabled}
      disabledReason={controls.openSessionFolder.reason}
      onclick={() => appState.openSessionFolder()}
      >{copy.openSessionFolder}</Button
    >
    <Button
      variant="primary"
      disabled={!controls.startAnother.enabled}
      disabledReason={controls.startAnother.reason}
      onclick={() => appState.startAnotherSession()}>{copy.startAnother}</Button
    >
  </div>
{/if}

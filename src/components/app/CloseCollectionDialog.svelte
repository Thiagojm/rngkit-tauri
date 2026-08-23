<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';
  import Dialog from '../ui/Dialog.svelte';

  const open = $derived(appState.closePrompt !== 'none');
  const finalizing = $derived(appState.closePrompt === 'finalizing');
</script>

<Dialog
  title={finalizing ? copy.close.finalizingTitle : copy.close.title}
  {open}
  dismissible={!finalizing}
  onClose={() => appState.keepCollecting()}
>
  <p>{finalizing ? copy.close.finalizingBody : copy.close.body}</p>
  {#snippet actions()}
    {#if finalizing}
      <Button disabled disabledReason={copy.close.finalizingReason}
        >{copy.close.stopAndExit}</Button
      >
    {:else}
      <Button onclick={() => appState.keepCollecting()}
        >{copy.close.keep}</Button
      >
      <Button variant="primary" onclick={() => appState.stopAndExit()}
        >{copy.close.stopAndExit}</Button
      >
    {/if}
  {/snippet}
</Dialog>

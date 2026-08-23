<script lang="ts">
  import Button from '../ui/Button.svelte';
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import SourceCandidate from './SourceCandidate.svelte';

  const collection = $derived(appState.snapshot.collection);
  const controls = $derived(appState.controls);
</script>

<Button
  disabled={!controls.refresh.enabled}
  disabledReason={controls.refresh.reason}
  onclick={() => void appState.refreshSources()}>{copy.refreshSources}</Button
>
<fieldset class="flex flex-col gap-2" disabled={!controls.configure.enabled}>
  <legend class="text-sm font-medium">{copy.entropySource}</legend>
  {#if collection.candidates.length === 0}
    <p class="text-sm text-text-muted">{copy.noSources}</p>
  {:else}
    {#each collection.candidates as candidate (candidate.token)}
      <SourceCandidate
        {candidate}
        selected={collection.selectedToken === candidate.token}
        disabled={!controls.configure.enabled}
        onSelect={(token) => appState.selectSource(token)}
      />
    {/each}
  {/if}
</fieldset>
{#if collection.familyWarning}
  <p class="text-sm text-text-muted" role="note">{collection.familyWarning}</p>
{/if}

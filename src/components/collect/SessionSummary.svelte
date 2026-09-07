<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import { candidateLabel } from '../../state/controls';

  const collection = $derived(appState.snapshot.collection);
  const controls = $derived(appState.controls);
  const selected = $derived(
    collection.candidates.find(
      (candidate) => candidate.token === collection.selectedToken,
    ),
  );
</script>

{#if selected}
  <p class="text-sm text-text-muted">
    Selected source: {candidateLabel(selected)}
    {#if controls.showFold}
      · Fold {collection.fold === 0 ? copy.fold.raw : String(collection.fold)}
    {/if}
  </p>
{/if}
{#if collection.sessionStem}
  <p class="font-mono text-sm text-text-muted">{collection.sessionStem}</p>
{/if}

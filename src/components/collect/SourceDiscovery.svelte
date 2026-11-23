<script lang="ts">
  import Button from '../ui/Button.svelte';
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import SourceCandidate from './SourceCandidate.svelte';

  const collection = $derived(appState.snapshot.collection);
  const controls = $derived(appState.controls);
  // configure is disabled exactly while a session is collecting or stopping.
  const locked = $derived(!controls.configure.enabled);
</script>

<div class="flex flex-wrap items-center gap-2">
  <Button
    disabled={!controls.refresh.enabled}
    describedBy={locked ? 'session-lock-note' : 'refresh-reason'}
    onclick={() => void appState.refreshSources()}>{copy.refreshSources}</Button
  >
  <button
    type="button"
    class="rounded-sm px-1 py-1 text-sm font-medium text-text-muted hover:bg-surface-muted hover:text-text"
    onclick={() => appState.openDeviceSetup()}>{copy.deviceSetup}</button
  >
</div>
<!-- Reasons sit below the row so Refresh sources and Device setup never move. -->
{#if locked}
  <p id="session-lock-note" class="text-sm text-text-muted" role="note">
    {collection.state === 'stopping'
      ? copy.sessionLocked.stopping
      : copy.sessionLocked.collecting}
  </p>
{:else if !controls.refresh.enabled}
  <p id="refresh-reason" class="text-sm text-text-muted">
    {controls.refresh.reason}
  </p>
{/if}
<fieldset
  class="flex flex-col gap-2"
  disabled={locked}
  aria-describedby={locked ? 'session-lock-note' : undefined}
>
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

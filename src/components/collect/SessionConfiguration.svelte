<script lang="ts">
  import { copy, FOLD_OPTIONS } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';
  import Field from '../ui/Field.svelte';
  import CollectionControls from './CollectionControls.svelte';
  import SourceDiscovery from './SourceDiscovery.svelte';

  const collection = $derived(appState.snapshot.collection);
  const controls = $derived(appState.controls);
  const preferencesWarning = $derived(appState.snapshot.preferencesWarning);

  function onBitsChange(event: Event): void {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    appState.setSampleBits(value);
  }

  function onIntervalChange(event: Event): void {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    appState.setIntervalSeconds(value);
  }

  function onFoldChange(event: Event): void {
    const value = Number((event.currentTarget as HTMLSelectElement).value);
    appState.setFold(value);
  }
</script>

<section
  class="flex flex-col gap-4 rounded-md border border-border p-4"
  aria-labelledby="collect-config"
>
  <h2 id="collect-config" class="text-lg font-medium">Session</h2>
  <SourceDiscovery />
  {#if preferencesWarning}
    <p class="text-sm text-text-muted" role="note">{preferencesWarning}</p>
  {/if}
  <Field id="sample-bits" label={copy.sampleBits}>
    <input
      id="sample-bits"
      class="w-full rounded-md border border-border bg-surface px-2 py-1"
      type="number"
      min="8"
      step="8"
      value={collection.sampleBits}
      disabled={!controls.configure.enabled}
      onchange={onBitsChange}
    />
  </Field>
  <Field id="sample-interval" label={copy.sampleInterval}>
    <input
      id="sample-interval"
      class="w-full rounded-md border border-border bg-surface px-2 py-1"
      type="number"
      min="1"
      step="1"
      value={collection.intervalSeconds}
      disabled={!controls.configure.enabled}
      onchange={onIntervalChange}
    />
  </Field>
  {#if controls.showFold}
    <Field id="fold" label={copy.fold.label}>
      <select
        id="fold"
        class="w-full rounded-md border border-border bg-surface px-2 py-1"
        value={collection.fold ?? 0}
        disabled={!controls.configure.enabled}
        onchange={onFoldChange}
      >
        {#each FOLD_OPTIONS as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </Field>
  {/if}
  <Field
    id="output-root"
    label={copy.outputRoot}
    hint={collection.outputRootLabel ?? copy.noOutputFolder}
    group
  >
    <Button
      disabled={!controls.chooseFolder.enabled}
      disabledReason={controls.chooseFolder.reason}
      onclick={() => appState.chooseOutputFolder()}>{copy.chooseFolder}</Button
    >
    <Button
      disabled={!controls.openCollectionWorkingFolder.enabled}
      disabledReason={controls.openCollectionWorkingFolder.reason}
      onclick={() => appState.openCollectionWorkingFolder()}
      >{copy.openWorkingFolder}</Button
    >
  </Field>
  <CollectionControls />
</section>

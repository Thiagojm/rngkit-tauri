<script lang="ts">
  import { tick } from 'svelte';
  import { copy, FOLD_OPTIONS } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';
  import Dialog from '../ui/Dialog.svelte';
  import Field from '../ui/Field.svelte';
  import CollectionControls from './CollectionControls.svelte';
  import SourceDiscovery from './SourceDiscovery.svelte';

  const collection = $derived(appState.snapshot.collection);
  const controls = $derived(appState.controls);
  const preferencesWarning = $derived(appState.snapshot.preferencesWarning);

  let bits = $state('');
  let bitsEdited = $state(false);
  let interval = $state('');
  let intervalEdited = $state(false);
  $effect(() => {
    if (!bitsEdited) bits = String(collection.sampleBits);
  });
  $effect(() => {
    if (!intervalEdited) interval = String(collection.intervalSeconds);
  });
  let submitting = $state(false);
  let invalidField = $state<'bits' | 'interval' | null>(null);

  async function start(): Promise<void> {
    const sampleBits = Number(bits);
    const seconds = Number(interval);
    if (
      !Number.isInteger(sampleBits) ||
      sampleBits < 8 ||
      sampleBits % 8 !== 0 ||
      sampleBits > 4294967295
    ) {
      invalidField = 'bits';
      return;
    }
    if (!Number.isInteger(seconds) || seconds <= 0 || seconds > 4294967295) {
      invalidField = 'interval';
      return;
    }
    if (submitting) return;
    submitting = true;
    try {
      await appState.startWithSettings(sampleBits, seconds);
    } finally {
      submitting = false;
    }
  }

  async function closeValidation(): Promise<void> {
    if (invalidField === null) return;
    const id = invalidField === 'bits' ? 'sample-bits' : 'sample-interval';
    invalidField = null;
    await tick();
    document.getElementById(id)?.focus();
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
      value={bits}
      disabled={!controls.configure.enabled || submitting}
      oninput={(event) => {
        bitsEdited = true;
        bits = event.currentTarget.value;
      }}
    />
  </Field>
  <Field id="sample-interval" label={copy.sampleInterval}>
    <input
      id="sample-interval"
      class="w-full rounded-md border border-border bg-surface px-2 py-1"
      type="number"
      min="1"
      step="1"
      value={interval}
      disabled={!controls.configure.enabled || submitting}
      oninput={(event) => {
        intervalEdited = true;
        interval = event.currentTarget.value;
      }}
    />
  </Field>
  {#if controls.showFold}
    <Field id="fold" label={copy.fold.label}>
      <select
        id="fold"
        class="w-full rounded-md border border-border bg-surface px-2 py-1"
        value={collection.fold ?? 0}
        disabled={!controls.configure.enabled || submitting}
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
  <CollectionControls onStart={start} {submitting} />
</section>

<Dialog
  title="Cannot start collection"
  open={invalidField !== null}
  onClose={closeValidation}
>
  {#if invalidField === 'bits'}
    <p>
      Sample size must be a positive whole number divisible by 8 because each
      byte contains 8 bits. Enter a value such as 8, 16, 1024, or 2048.
      Collection has not started.
    </p>
  {:else}
    <p>
      Sample interval must be a whole number greater than 0 seconds, such as 1,
      2, or 10. Collection has not started.
    </p>
  {/if}
  <p class="mt-2">The maximum supported value is 4,294,967,295.</p>
  {#snippet actions()}
    <Button onclick={closeValidation}
      >Edit {invalidField === 'bits'
        ? 'sample size'
        : 'sample interval'}</Button
    >
  {/snippet}
</Dialog>

<script lang="ts">
  import { tick } from 'svelte';
  import { copy, FOLD_OPTIONS } from '../../copy';
  import { appState } from '../../state/app-state.svelte';

  const combine = $derived(appState.snapshot.combine);
  const controls = $derived(appState.controls);

  function foldLabel(fold: number | null): string {
    if (fold === null) {
      return '—';
    }
    return (
      FOLD_OPTIONS.find((option) => option.value === fold)?.label ??
      String(fold)
    );
  }

  function formatLabel(format: string): string {
    if (format === 'current_csv') return 'Current CSV';
    if (format === 'legacy_v3_csv') return 'Legacy v3 CSV';
    return 'Unknown';
  }

  const caption = 'CSV inputs in derived chronological order';
  let width = $state(0);
  let inputList: HTMLDivElement | undefined = $state();
  const wide = $derived(
    width >=
      60 * parseFloat(getComputedStyle(document.documentElement).fontSize),
  );
  let renderedWide = false;

  $effect.pre(() => {
    if (wide === renderedWide) return;
    renderedWide = wide;

    const active = document.activeElement;
    if (
      !(active instanceof HTMLButtonElement) ||
      !inputList?.contains(active)
    ) {
      return;
    }
    const inputId = active.dataset.inputId;
    if (!inputId) return;

    void tick().then(() => {
      if (document.activeElement === document.body) {
        inputList
          ?.querySelector<HTMLButtonElement>(
            `[data-input-id="${CSS.escape(inputId)}"]`,
          )
          ?.focus();
      }
    });
  });
</script>

{#snippet remove(row: (typeof combine.inputs)[number])}
  <button
    class="text-sm text-status-failed underline disabled:text-text-muted disabled:no-underline"
    type="button"
    data-input-id={row.inputId}
    disabled={!controls.combine.enabled}
    aria-label={`${copy.removeCombineInput} ${row.basename} #${row.ordinal}`}
    onclick={() => appState.removeCombineInput(row.inputId)}
    >{copy.removeCombineInput}</button
  >
{/snippet}

{#snippet stateLabel(row: (typeof combine.inputs)[number])}
  {#if row.valid}
    Valid
  {:else}
    {row.error}
  {/if}
{/snippet}

{#if combine.inputs.length === 0}
  <p class="text-sm text-text-muted">{copy.noCombineInputs}</p>
{:else}
  <div
    class="min-w-0 overflow-x-auto"
    bind:this={inputList}
    bind:clientWidth={width}
    data-testid="combine-inputs"
  >
    {#if !wide}
      <ol class="flex flex-col gap-2" aria-label={caption}>
        {#each combine.inputs as row (row.inputId)}
          <li class="rounded-md border border-border p-3 text-sm">
            <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
              <span class="text-text-muted">#{row.ordinal}</span>
              <span class="min-w-0 flex-1 break-all font-mono"
                >{row.basename}</span
              >
              <span>{@render stateLabel(row)}</span>
              {@render remove(row)}
            </div>
            <dl
              class="mt-2 grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1"
            >
              <dt class="text-text-muted">Format</dt>
              <dd>{formatLabel(row.format)}</dd>
              <dt class="text-text-muted">Source</dt>
              <dd>{row.source}</dd>
              <dt class="text-text-muted">Bits</dt>
              <dd>{row.sampleBits}</dd>
              <dt class="text-text-muted">Interval</dt>
              <dd>{row.intervalSeconds}</dd>
              <dt class="text-text-muted">{copy.fold.label}</dt>
              <dd>{foldLabel(row.fold)}</dd>
              <dt class="text-text-muted">Time range</dt>
              <dd class="break-all font-mono">
                {row.firstTimestamp} → {row.lastTimestamp}
              </dd>
              <dt class="text-text-muted">Rows</dt>
              <dd>{row.rows}</dd>
            </dl>
          </li>
        {/each}
      </ol>
    {:else}
      <table class="w-full border-collapse text-left text-sm">
        <caption class="sr-only">{caption}</caption>
        <thead>
          <tr class="border-b border-border">
            <th class="px-2 py-2 font-medium">#</th>
            <th class="px-2 py-2 font-medium">Basename</th>
            <th class="px-2 py-2 font-medium">Format</th>
            <th class="px-2 py-2 font-medium">Source</th>
            <th class="px-2 py-2 font-medium">Bits</th>
            <th class="px-2 py-2 font-medium">Interval</th>
            <th class="px-2 py-2 font-medium">{copy.fold.label}</th>
            <th class="px-2 py-2 font-medium">Time range</th>
            <th class="px-2 py-2 font-medium">Rows</th>
            <th class="px-2 py-2 font-medium">State</th>
            <th class="px-2 py-2 font-medium"
              ><span class="sr-only">Action</span></th
            >
          </tr>
        </thead>
        <tbody>
          {#each combine.inputs as row (row.inputId)}
            <tr class="border-b border-border">
              <td class="whitespace-nowrap px-2 py-2">{row.ordinal}</td>
              <td class="break-all px-2 py-2 font-mono">{row.basename}</td>
              <td class="px-2 py-2">{formatLabel(row.format)}</td>
              <td class="px-2 py-2">{row.source}</td>
              <td class="whitespace-nowrap px-2 py-2">{row.sampleBits}</td>
              <td class="whitespace-nowrap px-2 py-2">{row.intervalSeconds}</td>
              <td class="whitespace-nowrap px-2 py-2">{foldLabel(row.fold)}</td>
              <td class="whitespace-nowrap px-2 py-2 font-mono">
                <span class="block">{row.firstTimestamp}</span>
                <span class="block">{row.lastTimestamp}</span>
              </td>
              <td class="whitespace-nowrap px-2 py-2">{row.rows}</td>
              <td class="px-2 py-2">{@render stateLabel(row)}</td>
              <td class="px-2 py-2">{@render remove(row)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
{/if}

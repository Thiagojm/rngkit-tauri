<script lang="ts">
  import Button from '../components/ui/Button.svelte';
  import Field from '../components/ui/Field.svelte';
  import MetricCard from '../components/ui/MetricCard.svelte';
  import StatusBanner from '../components/ui/StatusBanner.svelte';
  import { copy, FOLD_OPTIONS } from '../copy';
  import { appState } from '../state/app-state.svelte';
  import { candidateLabel } from '../state/controls';

  const collection = $derived(appState.snapshot.collection);
  const controls = $derived(appState.controls);
  const selected = $derived(
    collection.candidates.find(
      (candidate) => candidate.token === collection.selectedToken,
    ),
  );
</script>

<div class="flex flex-col gap-4">
  <h1 class="text-2xl font-semibold">{copy.destinations.collect}</h1>
  <div class="@container">
    <div
      class="grid grid-cols-1 gap-4 @[36rem]:grid-cols-[minmax(16rem,20rem)_minmax(0,1fr)]"
    >
      <section
        class="flex flex-col gap-4 rounded-md border border-border p-4"
        aria-labelledby="collect-config"
      >
        <h2 id="collect-config" class="text-lg font-medium">Session</h2>
        <Button
          disabled={!controls.refresh.enabled}
          disabledReason={controls.refresh.reason}>{copy.refreshSources}</Button
        >
        <fieldset
          class="flex flex-col gap-2"
          disabled={!controls.configure.enabled}
        >
          <legend class="text-sm font-medium">{copy.entropySource}</legend>
          {#if collection.candidates.length === 0}
            <p class="text-sm text-text-muted">{copy.noSources}</p>
          {:else}
            {#each collection.candidates as candidate (candidate.token)}
              <label class="flex items-center gap-2 text-sm">
                <input
                  type="radio"
                  name="entropy-source"
                  value={candidate.token}
                  checked={collection.selectedToken === candidate.token}
                  disabled={!controls.configure.enabled}
                  onchange={() => appState.selectSource(candidate.token)}
                />
                {candidateLabel(candidate)}
              </label>
            {/each}
          {/if}
        </fieldset>
        {#if collection.familyWarning}
          <p class="text-sm text-text-muted" role="note">
            {collection.familyWarning}
          </p>
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
          />
        </Field>
        {#if controls.showFold}
          <Field id="fold" label={copy.fold.label}>
            <select
              id="fold"
              class="w-full rounded-md border border-border bg-surface px-2 py-1"
              value={collection.fold ?? 0}
              disabled={!controls.configure.enabled}
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
          hint={collection.outputRootLabel ?? 'No output folder selected.'}
          group
        >
          <Button
            disabled={!controls.chooseFolder.enabled}
            disabledReason={controls.chooseFolder.reason}
            >{copy.chooseFolder}</Button
          >
        </Field>
        {#if controls.showStart}
          <Button
            variant="primary"
            disabled={!controls.start.enabled}
            disabledReason={controls.start.reason}>{copy.start}</Button
          >
        {/if}
        {#if controls.showStop}
          <Button
            variant="primary"
            disabled={!controls.stop.enabled}
            disabledReason={controls.stop.reason}>{copy.stop}</Button
          >
        {/if}
      </section>

      <section
        class="flex min-w-0 flex-col gap-4 rounded-md border border-border p-4"
        aria-labelledby="collect-monitor"
      >
        <h2 id="collect-monitor" class="text-lg font-medium">Monitoring</h2>
        <StatusBanner
          state={collection.state}
          label={collection.statusLabel}
          detail={collection.errorMessage}
        />
        <div class="grid grid-cols-2 gap-3 @min-[28rem]:grid-cols-3">
          <MetricCard label="Samples" value={String(collection.sampleCount)} />
          <MetricCard label="Elapsed" value={collection.elapsedLabel} />
          <MetricCard
            label="Observed one proportion"
            value={collection.onesProportionLabel}
          />
          <MetricCard
            label="Descriptive cumulative Z"
            value={collection.cumulativeZLabel}
          />
          <MetricCard
            label="Overruns"
            value={String(collection.overrunCount)}
          />
        </div>
        <div
          class="flex min-h-48 items-center rounded-md border border-dashed border-border px-4 py-8 text-sm text-text-muted"
          role="img"
          aria-label="Cumulative Z chart"
        >
          {copy.chartPlaceholder}
        </div>
        <p
          class="rounded-md border border-border bg-surface-muted px-4 py-3 text-sm text-text-muted"
        >
          {copy.statsWarning}
        </p>
        {#if selected}
          <p class="text-sm text-text-muted">
            Selected source: {candidateLabel(selected)}
            {#if controls.showFold}
              · Fold {collection.fold === 0
                ? copy.fold.raw
                : String(collection.fold)}
            {/if}
          </p>
        {/if}
        {#if collection.sessionStem}
          <p class="font-mono text-sm text-text-muted">
            {collection.sessionStem}
          </p>
        {/if}
        {#if controls.showTerminalActions}
          <div class="flex flex-wrap gap-2">
            <Button
              disabled={!controls.openSessionFolder.enabled}
              disabledReason={controls.openSessionFolder.reason}
              >{copy.openSessionFolder}</Button
            >
            <Button
              variant="primary"
              disabled={!controls.startAnother.enabled}
              disabledReason={controls.startAnother.reason}
              >{copy.startAnother}</Button
            >
          </div>
        {/if}
      </section>
    </div>
  </div>
</div>

<script lang="ts">
  import SessionConfiguration from '../components/collect/SessionConfiguration.svelte';
  import SessionSummary from '../components/collect/SessionSummary.svelte';
  import MetricCard from '../components/ui/MetricCard.svelte';
  import StatusBanner from '../components/ui/StatusBanner.svelte';
  import { copy } from '../copy';
  import { appState } from '../state/app-state.svelte';

  const collection = $derived(appState.snapshot.collection);
</script>

<div class="flex flex-col gap-4">
  <h1 class="text-2xl font-semibold">{copy.destinations.collect}</h1>
  <div class="@container">
    <div
      class="grid grid-cols-1 gap-4 @[36rem]:grid-cols-[minmax(16rem,20rem)_minmax(0,1fr)]"
    >
      <SessionConfiguration />

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
        <SessionSummary />
      </section>
    </div>
  </div>
</div>

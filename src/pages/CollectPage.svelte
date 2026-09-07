<script lang="ts">
  import LiveZChart from '../components/collect/LiveZChart.svelte';
  import SessionConfiguration from '../components/collect/SessionConfiguration.svelte';
  import ErrorPanel from '../components/ui/ErrorPanel.svelte';
  import MetricCard from '../components/ui/MetricCard.svelte';
  import StatusBanner from '../components/ui/StatusBanner.svelte';
  import { copy } from '../copy';
  import { appState } from '../state/app-state.svelte';

  const collection = $derived(appState.snapshot.collection);
</script>

<div class="flex flex-col gap-2">
  <h1 class="text-2xl font-semibold">{copy.destinations.collect}</h1>
  <div class="@container">
    <div
      class="grid grid-cols-1 gap-4 @[36rem]:grid-cols-[minmax(16rem,20rem)_minmax(0,1fr)]"
    >
      <SessionConfiguration />

      <section
        class="flex min-w-0 flex-col gap-2 self-start rounded-md border border-border p-4"
        aria-labelledby="collect-monitor"
      >
        <div class="flex flex-wrap items-center justify-between gap-2">
          <h2 id="collect-monitor" class="text-lg font-medium">Monitoring</h2>
          <StatusBanner
            compact
            state={collection.state}
            label={collection.statusLabel}
            detail={collection.errorMessage}
          />
        </div>
        <ErrorPanel />
        <div class="grid grid-cols-2 gap-2 @min-[28rem]:grid-cols-3">
          <MetricCard
            compact
            label="Samples"
            value={String(collection.sampleCount)}
          />
          <MetricCard compact label="Elapsed" value={collection.elapsedLabel} />
          <MetricCard
            compact
            label="Observed one proportion"
            value={collection.onesProportionLabel}
          />
          <MetricCard
            compact
            label="Descriptive cumulative Z"
            value={collection.cumulativeZLabel}
          />
          <MetricCard
            compact
            label="Overruns"
            value={String(collection.overrunCount)}
          />
        </div>
        <LiveZChart />
      </section>
    </div>
  </div>
</div>

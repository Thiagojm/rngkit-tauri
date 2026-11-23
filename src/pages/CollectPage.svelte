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
  <div class="@container/collect">
    <div
      class="grid grid-cols-1 gap-4 @[52rem]/collect:grid-cols-[minmax(16rem,20rem)_minmax(0,1fr)]"
    >
      <SessionConfiguration />

      <section
        class="@container/monitor flex min-w-0 flex-col gap-2 self-start rounded-md border border-border p-3 @[52rem]/collect:p-4"
        aria-labelledby="collect-monitor"
      >
        <div class="flex flex-wrap items-center justify-between gap-2">
          <h2 id="collect-monitor" class="text-lg font-medium">Monitoring</h2>
          <StatusBanner
            compact
            state={collection.state}
            label={collection.statusLabel}
          />
        </div>
        <ErrorPanel />
        <div class="grid grid-cols-2 gap-2 @min-[28rem]/monitor:grid-cols-3">
          <MetricCard
            compact
            label={copy.metrics.samples}
            value={String(collection.sampleCount)}
          />
          <MetricCard
            compact
            label={copy.metrics.elapsed}
            value={collection.elapsedLabel}
          />
          <MetricCard
            compact
            label={copy.metrics.onesShare}
            value={collection.onesProportionLabel}
          />
          <MetricCard
            compact
            label={copy.metrics.cumulativeZ}
            value={collection.cumulativeZLabel}
          />
          <MetricCard
            compact
            label={copy.metrics.overruns}
            value={String(collection.overrunCount)}
          />
        </div>
        <LiveZChart />
      </section>
    </div>
  </div>
</div>

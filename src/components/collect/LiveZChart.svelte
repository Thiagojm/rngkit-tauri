<script lang="ts">
  import 'uplot/dist/uPlot.min.css';
  import { createChartAdapter } from '../../chart/uplot-adapter';
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import Button from '../ui/Button.svelte';

  let host: HTMLDivElement | undefined = $state();
  let following = $state(false);

  const collection = $derived(appState.snapshot.collection);
  const collectionActive = $derived(
    collection.state === 'collecting' || collection.state === 'stopping',
  );

  const adapter = createChartAdapter({
    labels: {
      series: copy.chart.series,
      xAxis: copy.chart.xAxis,
      yAxis: copy.chart.yAxis,
      zero: copy.chart.zero,
      refPlus: copy.chart.refPlus,
      refMinus: copy.chart.refMinus,
    },
    onViewportStateChange: (nextFollowing) => {
      following = nextFollowing;
    },
  });

  const chartSnapshot = $derived({
    version: appState.chartVersion,
    pointCount: appState.chartSeries.length,
  });
  const pointCount = $derived(chartSnapshot.pointCount);
  const empty = $derived(pointCount === 0);

  $effect(() => {
    const target = host;
    if (!target) {
      return;
    }
    adapter.mount(target);
    return () => adapter.destroy();
  });

  $effect(() => {
    const target = host;
    if (!target) {
      return;
    }
    target.dataset.theme = appState.theme;
    adapter.refreshTheme();
  });

  $effect(() => {
    if (host) {
      host.dataset.chartVersion = String(appState.chartVersion);
    }
    adapter.setData(appState.chartSeries.aligned(), collectionActive);
  });

  function fitAll(): void {
    adapter.fitAll(appState.chartSeries.aligned(), collectionActive);
  }
</script>

<section
  class="flex min-w-0 flex-col gap-2 rounded-md border border-border bg-surface-muted p-3"
  aria-labelledby="live-z-heading"
>
  <div class="flex flex-wrap items-start gap-3">
    <h3 id="live-z-heading" class="text-base font-semibold">
      {copy.chart.title}
    </h3>
    <div class="ms-auto">
      <Button disabled={empty} onclick={fitAll}>{copy.chart.fitAll}</Button>
    </div>
  </div>

  <div
    class="relative min-h-[18rem] min-w-0 rounded-sm border border-border bg-surface p-2 @[36rem]:min-h-[20rem] @[36rem]:h-[min(42vh,30rem)]"
  >
    <div
      bind:this={host}
      class="live-z-chart h-full min-h-[17rem] w-full min-w-0 @[36rem]:min-h-[19rem]"
      data-testid="live-z-chart"
      role="img"
      aria-label={copy.chart.caption}
    ></div>
    {#if empty}
      <p
        class="pointer-events-none absolute inset-0 flex items-center justify-center px-4 text-center text-sm text-text-muted"
      >
        {copy.chart.empty}
      </p>
    {/if}
  </div>
  <ul class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-text-muted">
    <li class="flex items-center gap-2">
      <span class="h-0.5 w-4 bg-chart-z" aria-hidden="true"></span>
      {copy.chart.series}
    </li>
    <li class="flex items-center gap-2">
      <span class="h-px w-4 bg-chart-zero" aria-hidden="true"></span>
      {copy.chart.zero}
    </li>
    <li class="flex items-center gap-2">
      <span
        class="w-4 border-t border-dashed border-chart-ref"
        aria-hidden="true"
      ></span>
      {copy.chart.refPlus}
    </li>
    <li class="flex items-center gap-2">
      <span
        class="w-4 border-t border-dashed border-chart-ref"
        aria-hidden="true"
      ></span>
      {copy.chart.refMinus}
    </li>
  </ul>
  <p class="text-sm text-text-muted" data-testid="chart-point-count">
    {copy.chart.retainedPoints}: {pointCount}
    · {following ? copy.chart.following : copy.chart.paused}
  </p>
</section>

<style>
  :global(.live-z-chart .uplot) {
    font-family: var(--font-sans);
    color: var(--color-text);
    width: 100%;
    height: 100%;
  }

  :global(.live-z-chart .u-wrap) {
    width: 100%;
  }
</style>

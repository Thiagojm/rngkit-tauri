import uPlot from 'uplot';
import { REFERENCE_Z } from './chart-data';

export type AlignedChartData = [number[], number[]];

export interface ChartLabels {
  series: string;
  xAxis: string;
  yAxis: string;
  zero: string;
  refPlus: string;
  refMinus: string;
}

export type ChartPlotCtor = typeof uPlot;

export interface ChartAdapterOptions {
  labels: ChartLabels;
  uPlot?: ChartPlotCtor;
  raf?: (callback: FrameRequestCallback) => number;
  caf?: (handle: number) => void;
  onViewportStateChange?: (following: boolean) => void;
}

export interface ChartAdapter {
  mount(target: HTMLElement): void;
  setData(data: AlignedChartData, collectionActive: boolean): void;
  fitAll(data: AlignedChartData, collectionActive: boolean): void;
  isFollowing(): boolean;
  refreshTheme(): void;
  destroy(): void;
}

function cssColor(el: HTMLElement, name: string, fallback: string): string {
  const value = getComputedStyle(el).getPropertyValue(name).trim();
  return value || fallback;
}

function emptyData(): AlignedChartData {
  return [[], []];
}

function referenceLinesPlugin(
  target: HTMLElement,
  labels: ChartLabels,
): uPlot.Plugin {
  return {
    hooks: {
      draw: [
        (plot) => {
          const { ctx, bbox } = plot;
          const yMin = plot.scales.y?.min;
          const yMax = plot.scales.y?.max;
          if (
            yMin == null ||
            yMax == null ||
            bbox.width <= 0 ||
            bbox.height <= 0
          ) {
            return;
          }
          const zero = cssColor(target, '--color-chart-zero', '#4a5a70');
          const ref = cssColor(target, '--color-chart-ref', '#b54708');
          ctx.save();
          ctx.beginPath();
          ctx.rect(bbox.left, bbox.top, bbox.width, bbox.height);
          ctx.clip();
          const drawLine = (
            yVal: number,
            color: string,
            dash: boolean,
            label: string,
          ) => {
            if (yVal < yMin || yVal > yMax) {
              return;
            }
            const y = plot.valToPos(yVal, 'y', true);
            ctx.strokeStyle = color;
            ctx.lineWidth = 1;
            ctx.setLineDash(dash ? [6, 4] : []);
            ctx.beginPath();
            ctx.moveTo(bbox.left, y);
            ctx.lineTo(bbox.left + bbox.width, y);
            ctx.stroke();
            ctx.setLineDash([]);
            ctx.fillStyle = color;
            ctx.font = '12px sans-serif';
            ctx.textBaseline = 'bottom';
            ctx.fillText(label, bbox.left + 6, y - 2);
          };
          drawLine(0, zero, false, labels.zero);
          drawLine(REFERENCE_Z, ref, true, labels.refPlus);
          drawLine(-REFERENCE_Z, ref, true, labels.refMinus);
          ctx.restore();
        },
      ],
    },
  };
}

function yRange(
  _plot: uPlot,
  dataMin: number,
  dataMax: number,
): uPlot.Range.MinMax {
  const min = Number.isFinite(dataMin)
    ? Math.min(dataMin, -REFERENCE_Z)
    : -REFERENCE_Z;
  const max = Number.isFinite(dataMax)
    ? Math.max(dataMax, REFERENCE_Z)
    : REFERENCE_Z;
  return uPlot.rangeNum(min, max, 0.1, true);
}

export function createChartAdapter(options: ChartAdapterOptions): ChartAdapter {
  const Plot = options.uPlot ?? uPlot;
  const raf = options.raf ?? ((cb) => requestAnimationFrame(cb));
  const caf = options.caf ?? ((id) => cancelAnimationFrame(id));
  let plot: uPlot | null = null;
  let target: HTMLElement | null = null;
  let pending: {
    data: AlignedChartData;
    resetScales: boolean;
    version: number;
  } | null = null;
  let lastData: AlignedChartData = emptyData();
  let lastCollectionActive = false;
  let following = true;
  let frameVersion = 0;
  let rafHandle = 0;
  let detachPointer: (() => void) | null = null;
  let resizeObserver: ResizeObserver | null = null;

  function notifyUserViewport(): void {
    if (following) {
      following = false;
      options.onViewportStateChange?.(false);
    }
    if (pending) {
      pending.resetScales = false;
    }
  }

  function flush(version: number): void {
    if (version !== frameVersion) {
      return;
    }
    rafHandle = 0;
    if (!plot || !pending) {
      return;
    }
    const next = pending;
    pending = null;
    plot.setData(next.data, next.resetScales);
  }

  function remember(data: AlignedChartData): void {
    lastData = data;
  }

  function schedule(data: AlignedChartData, resetScales: boolean): void {
    remember(data);
    const version = frameVersion;
    pending = {
      data,
      resetScales: Boolean(pending?.resetScales) || resetScales,
      version,
    };
    if (rafHandle !== 0) {
      return;
    }
    rafHandle = raf(() => {
      flush(version);
    });
  }

  function cancelPendingFrame(): void {
    frameVersion += 1;
    pending = null;
    if (rafHandle !== 0) {
      caf(rafHandle);
      rafHandle = 0;
    }
  }

  function attachPointer(instance: uPlot): () => void {
    const over = instance.over;
    let dragging = false;
    let zooming = false;
    let lastX = 0;
    let lastY = 0;
    let zoomStartX = 0;
    let zoomStartY = 0;

    const onDown = (event: MouseEvent) => {
      if (event.button !== 0) {
        return;
      }
      if (!event.shiftKey) {
        zooming = true;
        zoomStartX = event.clientX;
        zoomStartY = event.clientY;
        return;
      }
      event.preventDefault();
      event.stopImmediatePropagation();
      dragging = true;
      lastX = event.clientX;
      lastY = event.clientY;
    };
    const onMove = (event: MouseEvent) => {
      if (!dragging) {
        return;
      }
      event.preventDefault();
      const xMin = instance.scales.x?.min;
      const xMax = instance.scales.x?.max;
      const yMin = instance.scales.y?.min;
      const yMax = instance.scales.y?.max;
      if (xMin == null || xMax == null || yMin == null || yMax == null) {
        return;
      }
      const rect = over.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) {
        return;
      }
      const dx = event.clientX - lastX;
      const dy = event.clientY - lastY;
      lastX = event.clientX;
      lastY = event.clientY;
      const xSpan = xMax - xMin;
      const ySpan = yMax - yMin;
      instance.setScale('x', {
        min: xMin - (dx / rect.width) * xSpan,
        max: xMax - (dx / rect.width) * xSpan,
      });
      instance.setScale('y', {
        min: yMin + (dy / rect.height) * ySpan,
        max: yMax + (dy / rect.height) * ySpan,
      });
      notifyUserViewport();
    };
    const onUp = (event: MouseEvent) => {
      dragging = false;
      if (
        zooming &&
        (Math.abs(event.clientX - zoomStartX) > 2 ||
          Math.abs(event.clientY - zoomStartY) > 2)
      ) {
        notifyUserViewport();
      }
      zooming = false;
    };
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      const xMin = instance.scales.x?.min;
      const xMax = instance.scales.x?.max;
      if (xMin == null || xMax == null) {
        return;
      }
      const rect = over.getBoundingClientRect();
      if (rect.width <= 0) {
        return;
      }
      if (event.ctrlKey) {
        const factor = event.deltaY < 0 ? 0.9 : 1.1;
        const cursorX = instance.posToVal(event.clientX - rect.left, 'x');
        instance.setScale('x', {
          min: cursorX - (cursorX - xMin) * factor,
          max: cursorX + (xMax - cursorX) * factor,
        });
      } else {
        const shift = (event.deltaY / rect.width) * (xMax - xMin);
        instance.setScale('x', { min: xMin + shift, max: xMax + shift });
      }
      notifyUserViewport();
    };

    over.addEventListener('mousedown', onDown, true);
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    over.addEventListener('wheel', onWheel, { passive: false });
    return () => {
      over.removeEventListener('mousedown', onDown, true);
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
      over.removeEventListener('wheel', onWheel);
    };
  }

  function chartOptions(host: HTMLElement): uPlot.Options {
    return {
      width: Math.max(host.clientWidth, 320),
      height: Math.max(host.clientHeight, 288),
      class: 'live-z-uplot',
      legend: { show: false },
      cursor: {
        drag: { x: true, y: true, setScale: true },
      },
      scales: {
        x: { time: false },
        y: { range: yRange },
      },
      axes: [
        {
          label: options.labels.xAxis,
          stroke: () => cssColor(host, '--color-chart-axis', '#4a5a70'),
          grid: {
            stroke: () => cssColor(host, '--color-chart-grid', '#c5d0de'),
          },
          ticks: {
            stroke: () => cssColor(host, '--color-chart-grid', '#c5d0de'),
          },
          values: (_plot, splits) =>
            splits.map((value) => String(Math.round(value))),
        },
        {
          label: options.labels.yAxis,
          stroke: () => cssColor(host, '--color-chart-axis', '#4a5a70'),
          grid: {
            stroke: () => cssColor(host, '--color-chart-grid', '#c5d0de'),
          },
          ticks: {
            stroke: () => cssColor(host, '--color-chart-grid', '#c5d0de'),
          },
        },
      ],
      series: [
        {},
        {
          label: options.labels.series,
          stroke: () => cssColor(host, '--color-chart-z', '#1d6fd8'),
          width: 2.25,
          points: { show: false },
        },
      ],
      plugins: [referenceLinesPlugin(host, options.labels)],
    };
  }

  return {
    mount(host) {
      this.destroy();
      target = host;
      const mountedPlot = new Plot(chartOptions(host), emptyData(), host);
      plot = mountedPlot;
      detachPointer = attachPointer(mountedPlot);
      if (typeof ResizeObserver !== 'undefined') {
        resizeObserver = new ResizeObserver(() => {
          if (!plot || !target) {
            return;
          }
          const width = target.clientWidth;
          const height = target.clientHeight;
          if (width < 32 || height < 32) {
            return;
          }
          plot.setSize({ width, height });
        });
        resizeObserver.observe(host);
      }
      schedule(lastData, false);
    },
    setData(data, collectionActive) {
      const endedFollowingCollection =
        lastCollectionActive && !collectionActive;
      lastCollectionActive = collectionActive;
      if (endedFollowingCollection && following) {
        following = false;
        options.onViewportStateChange?.(false);
      }
      schedule(
        data,
        (following && collectionActive) || endedFollowingCollection,
      );
    },
    fitAll(data, collectionActive) {
      cancelPendingFrame();
      following = collectionActive;
      lastCollectionActive = collectionActive;
      options.onViewportStateChange?.(following);
      remember(data);
      if (!plot) {
        schedule(data, true);
        return;
      }
      plot.setData(data, true);
    },
    isFollowing() {
      return following;
    },
    refreshTheme() {
      plot?.redraw(false, true);
    },
    destroy() {
      cancelPendingFrame();
      detachPointer?.();
      detachPointer = null;
      resizeObserver?.disconnect();
      resizeObserver = null;
      plot?.destroy();
      plot = null;
      target = null;
    },
  };
}

import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  createChartAdapter,
  type ChartLabels,
  type ChartPlotCtor,
} from './uplot-adapter';

const labels: ChartLabels = {
  series: 'Descriptive cumulative Z',
  xAxis: 'Sample index',
  yAxis: 'Descriptive cumulative Z',
  zero: 'Zero',
  refPlus: 'Reference +1.96',
  refMinus: 'Reference -1.96',
};

class FakePlot {
  static instances: FakePlot[] = [];
  static opts: unknown[] = [];
  setData = vi.fn();
  setScale = vi.fn();
  setSize = vi.fn();
  destroy = vi.fn();
  valToPos = () => 0;
  posToVal = () => 0;
  root = document.createElement('div');
  over = document.createElement('div');
  ctx = {
    save() {},
    restore() {},
    beginPath() {},
    rect() {},
    clip() {},
    moveTo() {},
    lineTo() {},
    stroke() {},
    fillText() {},
    setLineDash() {},
  } as unknown as CanvasRenderingContext2D;
  bbox = { left: 0, top: 0, width: 100, height: 100 };
  scales = {
    x: { min: 1, max: 10 },
    y: { min: -2, max: 2 },
  };
  redraw = vi.fn();

  constructor(opts: unknown, _data: unknown, target: HTMLElement) {
    FakePlot.opts.push(opts);
    FakePlot.instances.push(this);
    this.root.className = 'uplot';
    this.over.className = 'u-over';
    this.root.append(this.over);
    target.append(this.root);
  }
}

function adapterHarness(onViewportStateChange = vi.fn()) {
  FakePlot.instances = [];
  FakePlot.opts = [];
  const frames: FrameRequestCallback[] = [];
  const host = document.createElement('div');
  document.body.append(host);
  const adapter = createChartAdapter({
    labels,
    uPlot: FakePlot as unknown as ChartPlotCtor,
    raf: (callback) => {
      frames.push(callback);
      return frames.length;
    },
    caf: () => {
      frames.length = 0;
    },
    onViewportStateChange,
  });
  return { adapter, host, frames, onViewportStateChange };
}

describe('uPlot adapter', () => {
  afterEach(() => {
    document.body.replaceChildren();
    vi.unstubAllGlobals();
  });

  it('constructs one plot, draws two series, and destroys it', () => {
    const { adapter, host, onViewportStateChange } = adapterHarness();
    adapter.mount(host);
    expect(FakePlot.instances).toHaveLength(1);
    expect(onViewportStateChange).not.toHaveBeenCalled();
    const opts = FakePlot.opts[0] as { series: unknown[] };
    expect(opts.series).toHaveLength(2);
    adapter.destroy();
    expect(FakePlot.instances[0]?.destroy).toHaveBeenCalledTimes(1);
  });

  it('coalesces append redraws onto one animation frame', () => {
    const { adapter, host, frames } = adapterHarness();
    adapter.mount(host);
    frames[0]?.(0);
    frames.length = 0;
    FakePlot.instances[0]?.setData.mockClear();
    adapter.setData([[1], [0.1]], true);
    adapter.setData(
      [
        [1, 2],
        [0.1, 0.2],
      ],
      true,
    );
    adapter.setData(
      [
        [1, 2, 3],
        [0.1, 0.2, 0.3],
      ],
      true,
    );
    expect(frames).toHaveLength(1);
    frames[0]?.(0);
    expect(FakePlot.instances[0]?.setData).toHaveBeenCalledTimes(1);
    expect(FakePlot.instances[0]?.setData.mock.calls[0]?.[0][0]).toEqual([
      1, 2, 3,
    ]);
    expect(FakePlot.instances[0]?.setData.mock.calls[0]?.[1]).toBe(true);
  });

  it('keeps scales while paused and resumes following from Fit all', () => {
    const { adapter, host, frames, onViewportStateChange } = adapterHarness();
    adapter.mount(host);
    frames[0]?.(0);
    frames.length = 0;
    FakePlot.instances[0]?.setData.mockClear();
    adapter.fitAll(
      [
        [1, 2],
        [0.1, 0.2],
      ],
      false,
    );
    FakePlot.instances[0]?.setData.mockClear();
    adapter.setData(
      [
        [1, 2],
        [0.1, 0.2],
      ],
      false,
    );
    frames[0]?.(0);
    expect(FakePlot.instances[0]?.setData.mock.calls[0]?.[1]).toBe(false);
    adapter.fitAll(
      [
        [1, 2],
        [0.1, 0.2],
      ],
      true,
    );
    expect(FakePlot.instances[0]?.setData.mock.calls.at(-1)?.[1]).toBe(true);
    expect(adapter.isFollowing()).toBe(true);
    expect(onViewportStateChange).toHaveBeenCalledWith(true);
  });

  it('cancels stale frames when Fit all supersedes a pending append', () => {
    const { adapter, host, frames } = adapterHarness();
    adapter.mount(host);
    frames[0]?.(0);
    frames.length = 0;
    FakePlot.instances[0]?.setData.mockClear();
    adapter.setData([[1], [0.1]], true);
    const staleFrame = frames[0];
    adapter.fitAll(
      [
        [1, 2],
        [0.1, 0.2],
      ],
      true,
    );
    staleFrame?.(0);
    expect(FakePlot.instances[0]?.setData).toHaveBeenCalledTimes(1);
    expect(FakePlot.instances[0]?.setData.mock.calls[0]?.[0]).toEqual([
      [1, 2],
      [0.1, 0.2],
    ]);
  });

  it('treats pointer zoom as a user viewport change', () => {
    const onViewportStateChange = vi.fn();
    const { adapter, host } = adapterHarness(onViewportStateChange);
    adapter.mount(host);
    FakePlot.instances[0]?.over.dispatchEvent(
      new MouseEvent('mousedown', {
        bubbles: true,
        button: 0,
        clientX: 10,
        clientY: 10,
      }),
    );
    window.dispatchEvent(
      new MouseEvent('mouseup', { clientX: 30, clientY: 10 }),
    );
    expect(onViewportStateChange).toHaveBeenCalledWith(false);
    expect(adapter.isFollowing()).toBe(false);
  });

  it('stops following and frames the final data when collection ends', () => {
    const { adapter, host, frames, onViewportStateChange } = adapterHarness();
    adapter.mount(host);
    frames[0]?.(0);
    frames.length = 0;
    FakePlot.instances[0]?.setData.mockClear();

    adapter.setData([[1], [0.1]], true);
    frames[0]?.(0);
    adapter.setData([[1], [0.1]], false);
    frames[0]?.(0);

    expect(adapter.isFollowing()).toBe(false);
    expect(onViewportStateChange).toHaveBeenCalledWith(false);
    expect(FakePlot.instances[0]?.setData.mock.calls.at(-1)?.[1]).toBe(true);
  });

  it('resizes the mounted plot without replacing it', () => {
    const resizeCallbacks: ResizeObserverCallback[] = [];
    class FakeResizeObserver {
      constructor(callback: ResizeObserverCallback) {
        resizeCallbacks.push(callback);
      }

      observe() {}

      disconnect() {}
    }
    vi.stubGlobal('ResizeObserver', FakeResizeObserver);
    const { adapter, host } = adapterHarness();
    Object.defineProperty(host, 'clientWidth', {
      configurable: true,
      value: 640,
    });
    Object.defineProperty(host, 'clientHeight', {
      configurable: true,
      value: 320,
    });
    adapter.mount(host);
    const plot = FakePlot.instances[0];

    resizeCallbacks[0]?.([], {} as ResizeObserver);

    expect(plot?.setSize).toHaveBeenCalledWith({ width: 640, height: 320 });
    expect(FakePlot.instances).toHaveLength(1);
  });

  it('refreshes theme colors without replacing the plot or viewport', () => {
    const onViewportStateChange = vi.fn();
    const { adapter, host } = adapterHarness(onViewportStateChange);
    adapter.mount(host);
    const plot = FakePlot.instances[0];

    adapter.refreshTheme();

    expect(FakePlot.instances).toHaveLength(1);
    expect(plot?.redraw).toHaveBeenCalledWith(false, true);
    expect(onViewportStateChange).not.toHaveBeenCalled();
  });
});

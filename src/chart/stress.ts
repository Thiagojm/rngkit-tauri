import { ChartSeries } from './chart-data';
import { syntheticCumulativeZ } from './synthetic';

export interface StressMeasurement {
  count: number;
  retainedCount: number;
  replaceMs: number;
  appendMs: number;
  heapDeltaBytes: number | null;
}

function heapUsed(): number | null {
  const usage = (
    globalThis as { process?: { memoryUsage?: () => { heapUsed: number } } }
  ).process?.memoryUsage?.();
  return usage?.heapUsed ?? null;
}

/** Measure replace-all plus one append. Does not render. */
export function measureSeriesStress(count: number): StressMeasurement {
  const heapBefore = heapUsed();
  const series = new ChartSeries();
  const data = syntheticCumulativeZ(count);
  const replaceStarted = performance.now();
  series.replaceAll(data.sampleIndex, data.cumulativeZ);
  const replaceMs = performance.now() - replaceStarted;
  const appendStarted = performance.now();
  series.append(count + 1, 0);
  const appendMs = performance.now() - appendStarted;
  const heapAfter = heapUsed();
  return {
    count,
    retainedCount: series.length,
    replaceMs,
    appendMs,
    heapDeltaBytes:
      heapBefore === null || heapAfter === null ? null : heapAfter - heapBefore,
  };
}

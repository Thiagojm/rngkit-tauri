import { describe, expect, it } from 'vitest';
import { ChartSeries, REFERENCE_Z } from './chart-data';
import { syntheticCumulativeZ } from './synthetic';

describe('ChartSeries', () => {
  it('appends aligned sample index and cumulative Z points', () => {
    const series = new ChartSeries();
    series.append(1, 0.5);
    series.append(2, -0.25);

    expect(series.length).toBe(2);
    expect(series.sampleIndex).toEqual([1, 2]);
    expect(series.cumulativeZ).toEqual([0.5, -0.25]);
    expect(series.aligned()[0]).toBe(series.sampleIndex);
    expect(series.aligned()[1]).toBe(series.cumulativeZ);
  });

  it('replaces and clears without leaving a second series', () => {
    const series = new ChartSeries();
    const seeded = syntheticCumulativeZ(4);
    series.replaceAll(seeded.sampleIndex, seeded.cumulativeZ);
    expect(series.length).toBe(4);
    expect(series.aligned()).toHaveLength(2);
    series.clear();
    expect(series.length).toBe(0);
    expect(series.sampleIndex).toEqual([]);
    expect(series.cumulativeZ).toEqual([]);
  });

  it('does not allocate per-point reference arrays', () => {
    expect(REFERENCE_Z).toBe(1.96);
    const series = new ChartSeries();
    series.replaceAll([1, 2, 3], [0, 0.1, -0.1]);
    expect(series.aligned()).toHaveLength(2);
  });
});

describe('syntheticCumulativeZ', () => {
  it('uses one-based sample indexes and the descriptive Z formula', () => {
    const { sampleIndex, cumulativeZ } = syntheticCumulativeZ(3);
    expect(sampleIndex).toEqual([1, 2, 3]);
    expect(cumulativeZ).toHaveLength(3);
    expect(cumulativeZ.every((value) => Number.isFinite(value))).toBe(true);
  });
});

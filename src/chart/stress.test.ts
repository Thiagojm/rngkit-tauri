import { describe, expect, it } from 'vitest';
import { measureSeriesStress } from './stress';

function report(
  label: string,
  measurement: ReturnType<typeof measureSeriesStress>,
) {
  const heap =
    measurement.heapDeltaBytes !== null
      ? `${(measurement.heapDeltaBytes / (1024 * 1024)).toFixed(1)} MiB heap delta`
      : 'heap unavailable';
  // Timing is recorded evidence, not a pass/fail threshold.
  console.info(
    `${label}: replace ${measurement.replaceMs.toFixed(2)}ms, append ${measurement.appendMs.toFixed(3)}ms, ${heap}`,
  );
}

describe('chart stress harness', () => {
  it('retains 100000 synthetic points', () => {
    const measurement = measureSeriesStress(100_000);
    expect(measurement.count).toBe(100_000);
    expect(measurement.retainedCount).toBe(100_001);
    expect(measurement.replaceMs).toBeGreaterThanOrEqual(0);
    expect(measurement.appendMs).toBeGreaterThanOrEqual(0);
    report('100000 points', measurement);
  });

  it('retains 1000000 synthetic points', () => {
    const measurement = measureSeriesStress(1_000_000);
    expect(measurement.count).toBe(1_000_000);
    expect(measurement.retainedCount).toBe(1_000_001);
    expect(measurement.replaceMs).toBeGreaterThanOrEqual(0);
    expect(measurement.appendMs).toBeGreaterThanOrEqual(0);
    report('1000000 points', measurement);
  });
});

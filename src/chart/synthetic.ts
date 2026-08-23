/** Deterministic (sample index, descriptive cumulative Z) series for tests. */

export function syntheticCumulativeZ(count: number): {
  sampleIndex: number[];
  cumulativeZ: number[];
} {
  const sampleIndex = new Array<number>(count);
  const cumulativeZ = new Array<number>(count);
  const bits = 8;
  let ones = 0;
  let seed = 1;
  for (let i = 0; i < count; i += 1) {
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    ones += seed % (bits + 1);
    const n = (i + 1) * bits;
    sampleIndex[i] = i + 1;
    cumulativeZ[i] = (2 * ones - n) / Math.sqrt(n);
  }
  return { sampleIndex, cumulativeZ };
}

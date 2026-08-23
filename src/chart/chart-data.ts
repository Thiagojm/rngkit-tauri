/** Aligned (sample index, cumulative Z) series for the active session. */

export const REFERENCE_Z = 1.96;

export class ChartSeries {
  sampleIndex: number[] = [];
  cumulativeZ: number[] = [];

  get length(): number {
    return this.sampleIndex.length;
  }

  append(sampleIndex: number, cumulativeZ: number): void {
    this.sampleIndex.push(sampleIndex);
    this.cumulativeZ.push(cumulativeZ);
  }

  replaceAll(sampleIndex: number[], cumulativeZ: number[]): void {
    this.sampleIndex = sampleIndex.slice();
    this.cumulativeZ = cumulativeZ.slice();
  }

  clear(): void {
    this.sampleIndex = [];
    this.cumulativeZ = [];
  }

  aligned(): [number[], number[]] {
    return [this.sampleIndex, this.cumulativeZ];
  }
}

import type { AppSnapshot, CollectionSnapshot } from './types';

export const SCENARIO_IDS = [
  'idle',
  'discovering',
  'ready',
  'collecting',
  'stopping',
  'completed',
  'failed',
  'reportsPreview',
  'reportsConflict',
  'combineCompatible',
  'combineIncompatible',
] as const;

export type ScenarioId = (typeof SCENARIO_IDS)[number];

export const SCENARIO_LABELS: Record<ScenarioId, string> = {
  idle: 'Idle',
  discovering: 'Discovering',
  ready: 'Ready',
  collecting: 'Collecting',
  stopping: 'Stopping',
  completed: 'Completed',
  failed: 'Failed',
  reportsPreview: 'Reports preview',
  reportsConflict: 'Reports conflict',
  combineCompatible: 'Combine compatible',
  combineIncompatible: 'Combine incompatible',
};

const BITB: CollectionSnapshot['candidates'][number] = {
  token: 'mock-bitb-1',
  familyLabel: 'BitBabbler',
  variant: 'White',
  ordinal: 1,
  requiresFold: true,
};

const PSEUDO: CollectionSnapshot['candidates'][number] = {
  token: 'mock-pseudo-1',
  familyLabel: 'PseudoRNG',
  ordinal: 1,
  requiresFold: false,
};

const emptyReports = { preview: null };
const emptyCombine = {
  inputs: [],
  compatible: false,
  incompatibility: null,
  result: null,
};

function collection(
  partial: Partial<CollectionSnapshot> &
    Pick<CollectionSnapshot, 'state' | 'statusLabel'>,
): CollectionSnapshot {
  return {
    candidates: [],
    selectedToken: null,
    familyWarning: null,
    sampleBits: 8,
    intervalSeconds: 1,
    fold: null,
    outputRootLabel: null,
    sampleCount: 0,
    elapsedLabel: '00:00:00',
    onesProportionLabel: '—',
    cumulativeZLabel: '—',
    overrunCount: 0,
    sessionStem: null,
    errorCode: null,
    errorMessage: null,
    ...partial,
  };
}

const configured: Partial<CollectionSnapshot> = {
  candidates: [BITB, PSEUDO],
  selectedToken: BITB.token,
  familyWarning: null,
  fold: 0,
  outputRootLabel: 'Chosen folder',
};

const live: Partial<CollectionSnapshot> = {
  ...configured,
  sampleCount: 12,
  elapsedLabel: '00:00:12',
  onesProportionLabel: '0.5104',
  cumulativeZLabel: '+0.72',
  overrunCount: 0,
  sessionStem: '20260822T101500_bitb_s8_i1_f0',
};

const nativePreview = {
  kindLabel: 'Native session',
  origin: 'Collected session',
  source: 'BitBabbler',
  sampleBits: 8,
  intervalSeconds: 1,
  fold: 0,
  status: 'Completed',
  rowCount: 12,
  warning: null,
  conflict: false,
};

const compatibleInputs = [
  {
    basename: '20260101T010000_bitb_s8_i1.csv',
    source: 'BitBabbler',
    sampleBits: 8,
    intervalSeconds: 1,
    fold: 0,
    firstTimestamp: '2026-01-01T01:00:00Z',
    lastTimestamp: '2026-01-01T01:00:10Z',
    rows: 10,
    valid: true,
    error: null,
  },
  {
    basename: '20260101T020000_bitb_s8_i1.csv',
    source: 'BitBabbler',
    sampleBits: 8,
    intervalSeconds: 1,
    fold: 0,
    firstTimestamp: '2026-01-01T02:00:00Z',
    lastTimestamp: '2026-01-01T02:00:08Z',
    rows: 8,
    valid: true,
    error: null,
  },
];

export const MOCK_SCENARIOS: Record<ScenarioId, AppSnapshot> = {
  idle: {
    collection: collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: emptyCombine,
  },
  discovering: {
    collection: collection({
      state: 'discovering',
      statusLabel: 'Discovering sources',
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: emptyCombine,
  },
  ready: {
    collection: collection({
      state: 'ready',
      statusLabel: 'Ready',
      ...configured,
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: emptyCombine,
  },
  collecting: {
    collection: collection({
      state: 'collecting',
      statusLabel: 'Collecting',
      ...live,
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: emptyCombine,
  },
  stopping: {
    collection: collection({
      state: 'stopping',
      statusLabel: 'Stopping',
      ...live,
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: emptyCombine,
  },
  completed: {
    collection: collection({
      state: 'completed',
      statusLabel: 'Completed',
      ...live,
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: emptyCombine,
  },
  failed: {
    collection: collection({
      state: 'failed',
      statusLabel: 'Failed',
      ...live,
      errorCode: 'source_unavailable',
      errorMessage: 'The selected source became unavailable.',
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: emptyCombine,
  },
  reportsPreview: {
    collection: collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    fileJob: 'idle',
    reports: { preview: nativePreview },
    combine: emptyCombine,
  },
  reportsConflict: {
    collection: collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    fileJob: 'idle',
    reports: { preview: { ...nativePreview, conflict: true } },
    combine: emptyCombine,
  },
  combineCompatible: {
    collection: collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: {
      inputs: compatibleInputs,
      compatible: true,
      incompatibility: null,
      result: {
        stem: '20260822T120000_concat_bitb_s8_i1_f0',
        inputCount: 2,
        totalRows: 18,
      },
    },
  },
  combineIncompatible: {
    collection: collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    fileJob: 'idle',
    reports: emptyReports,
    combine: {
      inputs: [
        compatibleInputs[0],
        {
          ...compatibleInputs[1],
          valid: false,
          error: 'Timestamp range overlaps the previous input.',
        },
      ],
      compatible: false,
      incompatibility:
        'Overlapping timestamp ranges are rejected, including equal boundaries.',
      result: null,
    },
  },
};

export const DEFAULT_SCENARIO: ScenarioId = 'idle';

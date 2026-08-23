import type { AppSnapshot, CollectionSnapshot, ThemePreference } from './types';

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
  sourceId: 'bitb',
  familyLabel: 'BitBabbler',
  variant: 'White',
  ordinal: 1,
  requiresFold: true,
};

const PSEUDO: CollectionSnapshot['candidates'][number] = {
  token: 'mock-pseudo-1',
  sourceId: 'pseudo',
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

function snapshot(
  collection: CollectionSnapshot,
  extras: Partial<
    Pick<AppSnapshot, 'fileJob' | 'reports' | 'combine' | 'theme'>
  > = {},
): AppSnapshot {
  return {
    collection,
    fileJob: extras.fileJob ?? 'idle',
    reports: extras.reports ?? emptyReports,
    combine: extras.combine ?? emptyCombine,
    theme: extras.theme ?? ('system' satisfies ThemePreference),
    preferencesWarning: null,
  };
}

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
    sessionId: null,
    lastEventSequence: 0,
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
  sessionId: 's1',
  lastEventSequence: 12,
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
  idle: snapshot(
    collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
  ),
  discovering: snapshot(
    collection({
      state: 'discovering',
      statusLabel: 'Discovering sources',
    }),
  ),
  ready: snapshot(
    collection({
      state: 'ready',
      statusLabel: 'Ready',
      ...configured,
    }),
  ),
  collecting: snapshot(
    collection({
      state: 'collecting',
      statusLabel: 'Collecting',
      ...live,
    }),
  ),
  stopping: snapshot(
    collection({
      state: 'stopping',
      statusLabel: 'Stopping',
      ...live,
    }),
  ),
  completed: snapshot(
    collection({
      state: 'completed',
      statusLabel: 'Completed',
      ...live,
    }),
  ),
  failed: snapshot(
    collection({
      state: 'failed',
      statusLabel: 'Failed',
      ...live,
      errorCode: 'source_unavailable',
      errorMessage: 'The selected source became unavailable.',
    }),
  ),
  reportsPreview: snapshot(
    collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    { reports: { preview: nativePreview } },
  ),
  reportsConflict: snapshot(
    collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    { reports: { preview: { ...nativePreview, conflict: true } } },
  ),
  combineCompatible: snapshot(
    collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    {
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
  ),
  combineIncompatible: snapshot(
    collection({
      state: 'idle',
      statusLabel: 'Idle',
    }),
    {
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
  ),
};

export const DEFAULT_SCENARIO: ScenarioId = 'idle';

/** Browser/dev discovery result. Nothing is selected automatically. */
export function browserDiscoverySnapshot(): AppSnapshot {
  return snapshot(
    collection({
      state: 'idle',
      statusLabel: 'Idle',
      candidates: [BITB, PSEUDO],
      selectedToken: null,
    }),
  );
}

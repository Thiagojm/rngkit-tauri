export const DESTINATIONS = ['collect', 'reports', 'combine', 'help'] as const;

export type Destination = (typeof DESTINATIONS)[number];

export const THEME_PREFERENCES = ['system', 'light', 'dark'] as const;

export type ThemePreference = (typeof THEME_PREFERENCES)[number];

export const COLLECTION_STATES = [
  'idle',
  'discovering',
  'ready',
  'collecting',
  'stopping',
  'completed',
  'failed',
] as const;

export type CollectionState = (typeof COLLECTION_STATES)[number];

export type FileJobState =
  'idle' | 'inspecting' | 'generating_report' | 'combining';

export interface SourceCandidateView {
  token: string;
  familyLabel: string;
  variant?: string;
  ordinal: number;
  requiresFold: boolean;
}

export interface CollectionSnapshot {
  state: CollectionState;
  statusLabel: string;
  candidates: SourceCandidateView[];
  selectedToken: string | null;
  familyWarning: string | null;
  sampleBits: number;
  intervalSeconds: number;
  fold: number | null;
  outputRootLabel: string | null;
  sampleCount: number;
  elapsedLabel: string;
  onesProportionLabel: string;
  cumulativeZLabel: string;
  overrunCount: number;
  sessionStem: string | null;
  errorCode: string | null;
  errorMessage: string | null;
}

export interface ReportPreview {
  kindLabel: string;
  origin: string;
  source: string;
  sampleBits: number;
  intervalSeconds: number;
  fold: number | null;
  status: string;
  rowCount: number;
  warning: string | null;
  conflict: boolean;
}

export interface CombineInputRow {
  basename: string;
  source: string;
  sampleBits: number;
  intervalSeconds: number;
  fold: number | null;
  firstTimestamp: string;
  lastTimestamp: string;
  rows: number;
  valid: boolean;
  error: string | null;
}

export interface CombineResult {
  stem: string;
  inputCount: number;
  totalRows: number;
}

export interface ReportsSnapshot {
  preview: ReportPreview | null;
}

export interface CombineSnapshot {
  inputs: CombineInputRow[];
  compatible: boolean;
  incompatibility: string | null;
  result: CombineResult | null;
}

export interface AppSnapshot {
  collection: CollectionSnapshot;
  fileJob: FileJobState;
  reports: ReportsSnapshot;
  combine: CombineSnapshot;
}

export interface Control {
  enabled: boolean;
  reason: string;
}

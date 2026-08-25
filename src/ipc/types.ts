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
  'idle' | 'inspecting' | 'generatingReport' | 'combining';

export const ERROR_CODES = [
  'invalid_configuration',
  'invalid_transition',
  'expired_selection',
  'source_unavailable',
  'source_busy',
  'source_disconnected',
  'source_timed_out',
  'permission_denied',
  'output_exists',
  'corrupt_input',
  'unsupported_input',
  'operation_conflict',
  'unexpected_failure',
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

export interface SafeErrorDto {
  code: ErrorCode;
  message: string;
  operationId?: string;
  recovery?: string;
}

export type CollectionEvent =
  | {
      kind: 'sessionStarted';
      sessionId: string;
      sequence: number;
      stem: string;
    }
  | {
      kind: 'sampleCommitted';
      sessionId: string;
      sequence: number;
      sampleIndex: number;
      sampleCount: number;
      elapsedLabel: string;
      onesProportionLabel: string;
      cumulativeZ: number;
      cumulativeZLabel: string;
    }
  | {
      kind: 'timingOverrun';
      sessionId: string;
      sequence: number;
      overrunCount: number;
    }
  | {
      kind: 'cleanStop';
      sessionId: string;
      sequence: number;
      sampleCount: number;
      overrunCount: number;
    }
  | {
      kind: 'terminalFailure';
      sessionId: string;
      sequence: number;
      code: ErrorCode;
      message: string;
      recovery?: string;
    };

export interface DiagnosticRecord {
  appVersion: string;
  libraryRevision: string;
  operationId: string;
  code: ErrorCode;
  detail: string;
}

export interface SourceCandidateView {
  token: string;
  sourceId: string;
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
  sessionId: string | null;
  lastEventSequence: number;
  errorCode: ErrorCode | null;
  errorMessage: string | null;
  errorRecovery: string | null;
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
  inputId: string;
  ordinal: number;
  basename: string;
  format: string;
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
  reportReady: boolean;
}

export interface CombineSnapshot {
  inputs: CombineInputRow[];
  compatible: boolean;
  incompatibility: string | null;
  result: CombineResult | null;
}

export type OutcomeSeverity = 'success' | 'warning' | 'error';
export type OutcomeOperation = 'collection' | 'report' | 'combine';
export type OutcomeActionId =
  | 'openSessionFolder'
  | 'openReport'
  | 'openReportFolder'
  | 'openDerivedFolder'
  | 'openCollectionWorkingFolder'
  | 'openReportWorkingFolder'
  | 'openCombineWorkingFolder';

export interface OutcomePathRow {
  label: string;
  path: string;
}

export interface OutcomeNotice {
  id: number;
  severity: OutcomeSeverity;
  operation: OutcomeOperation;
  title: string;
  message: string;
  paths: OutcomePathRow[];
  actions: OutcomeActionId[];
}

export type ClosePromptMode = 'confirm' | 'finalizing';

export const THEME_PREFERENCES = ['system', 'light', 'dark'] as const;

export type ThemePreference = (typeof THEME_PREFERENCES)[number];

export interface AppSnapshot {
  collection: CollectionSnapshot;
  fileJob: FileJobState;
  reports: ReportsSnapshot;
  combine: CombineSnapshot;
  theme: ThemePreference;
  preferencesWarning: string | null;
  diagnostics: DiagnosticRecord[];
  pendingOutcome: OutcomeNotice | null;
}

export {
  COLLECTION_STATES,
  THEME_PREFERENCES,
  type AppSnapshot,
  type CollectionEvent,
  type CollectionSnapshot,
  type CollectionState,
  type CombineInputRow,
  type CombineResult,
  type CombineSnapshot,
  type DiagnosticRecord,
  type FileJobState,
  type ReportPreview,
  type ReportsSnapshot,
  type SourceCandidateView,
  type ThemePreference,
} from '../ipc/types';

export type ClosePrompt = 'none' | 'confirm' | 'finalizing';

export const DESTINATIONS = ['collect', 'reports', 'combine', 'help'] as const;

export type Destination = (typeof DESTINATIONS)[number];

export interface Control {
  enabled: boolean;
  reason: string;
}

export {
  COLLECTION_STATES,
  type AppSnapshot,
  type CollectionSnapshot,
  type CollectionState,
  type CombineInputRow,
  type CombineResult,
  type CombineSnapshot,
  type FileJobState,
  type ReportPreview,
  type ReportsSnapshot,
  type SourceCandidateView,
} from '../ipc/types';

export const DESTINATIONS = ['collect', 'reports', 'combine', 'help'] as const;

export type Destination = (typeof DESTINATIONS)[number];

export const THEME_PREFERENCES = ['system', 'light', 'dark'] as const;

export type ThemePreference = (typeof THEME_PREFERENCES)[number];

export interface Control {
  enabled: boolean;
  reason: string;
}

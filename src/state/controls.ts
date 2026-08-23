import type { AppSnapshot, CollectionState, Control } from './types';

function enabled(): Control {
  return { enabled: true, reason: '' };
}

function disabled(reason: string): Control {
  return { enabled: false, reason };
}

function sessionBusy(state: CollectionState): boolean {
  return state === 'collecting' || state === 'stopping';
}

function startReason(state: CollectionState): string {
  switch (state) {
    case 'idle':
      return 'Select a source and valid session settings before starting.';
    case 'discovering':
      return 'Wait for source discovery to finish.';
    case 'ready':
      return '';
    case 'collecting':
      return 'A session is already collecting.';
    case 'stopping':
      return 'Wait for the current session to finish stopping.';
    case 'completed':
    case 'failed':
      return 'Use Start another session from the summary.';
  }
}

export interface DerivedControls {
  start: Control;
  stop: Control;
  startAnother: Control;
  refresh: Control;
  configure: Control;
  chooseFolder: Control;
  openSessionFolder: Control;
  reports: Control;
  combine: Control;
  generateReport: Control;
  createDerived: Control;
  showStart: boolean;
  showStop: boolean;
  showTerminalActions: boolean;
  showFold: boolean;
}

export function deriveControls(snapshot: AppSnapshot): DerivedControls {
  const { collection, reports, combine } = snapshot;
  const state = collection.state;
  const busy = sessionBusy(state);
  const selected = collection.candidates.find(
    (candidate) => candidate.token === collection.selectedToken,
  );

  const fileJobReason = busy
    ? 'File jobs cannot run while a session is collecting or stopping.'
    : '';

  return {
    start: state === 'ready' ? enabled() : disabled(startReason(state)),
    stop:
      state === 'collecting'
        ? enabled()
        : disabled(
            state === 'stopping'
              ? 'Stop is already in progress. Wait for the session to finalize.'
              : 'Stop is available only while collecting.',
          ),
    startAnother:
      state === 'completed' || state === 'failed'
        ? enabled()
        : disabled('Start another session is available after a session ends.'),
    refresh:
      busy || state === 'discovering'
        ? disabled(
            state === 'discovering'
              ? 'Source discovery is still running.'
              : 'Source discovery cannot run while a session is collecting or stopping.',
          )
        : enabled(),
    configure: busy
      ? disabled('Session settings cannot change during collection.')
      : enabled(),
    chooseFolder: busy
      ? disabled('The output folder cannot change during collection.')
      : enabled(),
    openSessionFolder:
      state === 'completed' || state === 'failed'
        ? enabled()
        : disabled('Open the session folder after collection finishes.'),
    reports: busy ? disabled(fileJobReason) : enabled(),
    combine: busy ? disabled(fileJobReason) : enabled(),
    generateReport:
      busy || !reports.preview
        ? disabled(
            busy
              ? fileJobReason
              : 'Inspect a session or file before generating a report.',
          )
        : enabled(),
    createDerived:
      busy || !combine.compatible || combine.inputs.length === 0
        ? disabled(
            busy
              ? fileJobReason
              : (combine.incompatibility ??
                  'Select compatible legacy v3 CSV files before creating a bundle.'),
          )
        : enabled(),
    showStart: state === 'idle' || state === 'discovering' || state === 'ready',
    showStop: state === 'collecting' || state === 'stopping',
    showTerminalActions: state === 'completed' || state === 'failed',
    showFold: Boolean(selected?.requiresFold),
  };
}

export function statusTone(state: CollectionState): string {
  switch (state) {
    case 'ready':
    case 'completed':
      return 'text-status-ready';
    case 'collecting':
    case 'stopping':
    case 'discovering':
      return 'text-status-collecting';
    case 'failed':
      return 'text-status-failed';
    default:
      return 'text-status-idle';
  }
}

export function candidateLabel(candidate: {
  familyLabel: string;
  variant?: string;
  ordinal: number;
}): string {
  const parts = [candidate.familyLabel];
  if (candidate.variant) {
    parts.push(candidate.variant);
  }
  parts.push(String(candidate.ordinal));
  return parts.join(' · ');
}

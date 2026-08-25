import { isTauri } from '@tauri-apps/api/core';
import {
  acknowledgeOutcome as requestAcknowledgeOutcome,
  applyDevScenario,
  chooseCsvInputs,
  clearCombineInputs,
  chooseOutputFolder,
  chooseReportInput,
  copyDiagnostics as requestCopyDiagnostics,
  createDerived,
  generateDerived,
  generateReport,
  getAppState,
  listenCloseRequested,
  openDerivedFolder,
  openCollectionWorkingFolder,
  openCombineWorkingFolder,
  openReport,
  openReportFolder,
  openReportWorkingFolder,
  openSessionFolder,
  removeCombineInput,
  refreshSources,
  replaceReport,
  safeErrorMessage,
  selectSource,
  setFold,
  setIntervalSeconds,
  setSampleBits,
  setTheme,
  startAnotherSession,
  startCollection,
  stopAndExit as requestStopAndExit,
  stopCollection,
} from '../ipc/client';
import { ChartSeries } from '../chart/chart-data';
import { syntheticCumulativeZ } from '../chart/synthetic';
import { copy } from '../copy';
import { deriveControls } from './controls';
import {
  DEFAULT_SCENARIO,
  MOCK_SCENARIOS,
  type ScenarioId,
} from './mock-scenarios';
import type { ClosePromptMode } from '../ipc/types';
import type {
  AppSnapshot,
  ClosePrompt,
  CollectionEvent,
  CollectionSnapshot,
  Destination,
  OutcomeActionId,
  OutcomeNotice,
  ThemePreference,
} from './types';

function cloneSnapshot(id: ScenarioId): AppSnapshot {
  return structuredClone(MOCK_SCENARIOS[id]);
}

const ACTIVE_RECONCILE_MS = 250;

function needsStartupDiscovery(snapshot: AppSnapshot): boolean {
  return (
    snapshot.collection.state === 'idle' &&
    snapshot.collection.candidates.length === 0 &&
    snapshot.collection.selectedToken === null
  );
}

function isActiveCollection(snapshot: AppSnapshot): boolean {
  return (
    snapshot.collection.state === 'collecting' ||
    snapshot.collection.state === 'stopping'
  );
}

export class AppViewState {
  destination = $state<Destination>('collect');
  theme = $state<ThemePreference>('system');
  scenarioId = $state<ScenarioId>(DEFAULT_SCENARIO);
  selectedToken = $state<string | null>(
    MOCK_SCENARIOS[DEFAULT_SCENARIO].collection.selectedToken,
  );
  replaceDialogOpen = $state(false);
  closePrompt = $state<ClosePrompt>('none');
  diagnosticsCopied = $state(false);
  backendSnapshot = $state<AppSnapshot>(cloneSnapshot(DEFAULT_SCENARIO));
  loadGeneration = 0;
  collectionChannelGeneration = 0;
  startupDiscoveryStarted = false;
  localOutcome = $state<OutcomeNotice | null>(null);
  dismissedOutcomeId = $state<number | null>(null);
  chartSeries = new ChartSeries();
  chartVersion = $state(0);

  snapshot = $derived({
    ...this.backendSnapshot,
    collection: {
      ...this.backendSnapshot.collection,
      selectedToken: this.selectedToken,
    },
  });
  controls = $derived(deriveControls(this.snapshot));
  outcome = $derived.by(() => {
    if (this.localOutcome) {
      return this.localOutcome;
    }
    const pending = this.backendSnapshot.pendingOutcome;
    return pending && pending.id !== this.dismissedOutcomeId ? pending : null;
  });

  reconcile(snapshot: AppSnapshot): void {
    if (
      this.localOutcome &&
      snapshot.pendingOutcome?.id !== this.localOutcome.id
    ) {
      this.localOutcome = null;
    }
    if (
      this.dismissedOutcomeId !== null &&
      snapshot.pendingOutcome?.id !== this.dismissedOutcomeId
    ) {
      this.dismissedOutcomeId = null;
    }
    this.backendSnapshot = snapshot;
    this.selectedToken = snapshot.collection.selectedToken;
    this.theme = snapshot.theme;
  }

  async reconcileCommandFailure(
    generation: number,
    fallback: AppSnapshot,
    error: unknown,
    warningField: 'familyWarning' | 'preferencesWarning' = 'familyWarning',
  ): Promise<void> {
    if (generation !== this.loadGeneration) {
      return;
    }
    let snapshot = fallback;
    try {
      snapshot = await getAppState();
    } catch {
      // Keep the last usable snapshot when state reconciliation also fails.
    }
    if (generation !== this.loadGeneration) {
      return;
    }
    const message = safeErrorMessage(error);
    if (warningField === 'preferencesWarning') {
      this.reconcile({
        ...snapshot,
        preferencesWarning: message,
      });
      return;
    }
    this.reconcile({
      ...snapshot,
      collection: {
        ...snapshot.collection,
        familyWarning: message,
      },
    });
  }

  async hydrate(): Promise<void> {
    const generation = ++this.loadGeneration;
    const snapshot = await getAppState();
    if (generation !== this.loadGeneration) {
      return;
    }
    this.reconcile(snapshot);
    if (!this.startupDiscoveryStarted && needsStartupDiscovery(snapshot)) {
      this.startupDiscoveryStarted = true;
      void this.refreshSources();
    }
    if (isActiveCollection(snapshot)) {
      void this.reconcileReloadedSession(generation);
    }
  }

  private async reconcileReloadedSession(generation: number): Promise<void> {
    while (
      generation === this.loadGeneration &&
      isActiveCollection(this.backendSnapshot)
    ) {
      await new Promise((resolve) => setTimeout(resolve, ACTIVE_RECONCILE_MS));
      if (generation !== this.loadGeneration) {
        return;
      }
      try {
        const latest = await getAppState();
        if (generation === this.loadGeneration) {
          this.reconcile(latest);
        }
      } catch {
        return;
      }
    }
  }

  async listenForClose(): Promise<() => void> {
    return listenCloseRequested((mode) => this.onCloseRequested(mode));
  }

  onCloseRequested(mode: ClosePromptMode): void {
    this.closePrompt = mode;
  }

  keepCollecting(): void {
    if (this.closePrompt === 'finalizing') {
      return;
    }
    this.closePrompt = 'none';
  }

  stopAndExit(): void {
    this.closePrompt = 'finalizing';
    if (isTauri()) {
      const generation = ++this.loadGeneration;
      const fallback = $state.snapshot(this.snapshot);
      this.reconcile({
        ...fallback,
        collection: {
          ...fallback.collection,
          state:
            fallback.collection.state === 'collecting'
              ? 'stopping'
              : fallback.collection.state,
          statusLabel:
            fallback.collection.state === 'collecting'
              ? 'Stopping'
              : fallback.collection.statusLabel,
        },
      });
      void requestStopAndExit()
        .then((snapshot) => {
          if (generation !== this.loadGeneration) {
            return;
          }
          const current = this.backendSnapshot.collection;
          if (
            snapshot.collection.sessionId === current.sessionId &&
            snapshot.collection.lastEventSequence < current.lastEventSequence
          ) {
            return;
          }
          this.reconcile(snapshot);
        })
        .catch((error: unknown) =>
          this.reconcileCommandFailure(generation, fallback, error),
        );
      return;
    }
    this.stopCollection();
    if (
      this.snapshot.collection.state === 'completed' ||
      this.snapshot.collection.state === 'failed'
    ) {
      this.closePrompt = 'none';
    }
  }

  async copyDiagnostics(): Promise<boolean> {
    try {
      const text = await requestCopyDiagnostics(this.backendSnapshot);
      if (!navigator.clipboard?.writeText) {
        this.diagnosticsCopied = false;
        return false;
      }
      await navigator.clipboard.writeText(text);
      this.diagnosticsCopied = true;
      return true;
    } catch {
      this.diagnosticsCopied = false;
      return false;
    }
  }

  applyScenario(id: ScenarioId): void {
    const generation = ++this.loadGeneration;
    this.scenarioId = id;
    this.replaceDialogOpen = false;
    this.closePrompt = 'none';
    this.diagnosticsCopied = false;
    this.localOutcome = null;
    this.dismissedOutcomeId = null;
    this.reconcile(cloneSnapshot(id));
    this.syncMockChart(this.backendSnapshot.collection);
    if (import.meta.env.DEV && isTauri()) {
      void applyDevScenario(id).then((snapshot) => {
        if (generation === this.loadGeneration && this.scenarioId === id) {
          this.reconcile(snapshot);
          this.syncMockChart(snapshot.collection);
        }
      });
    }
  }

  selectSource(token: string): void {
    if (
      !this.snapshot.collection.candidates.some(
        (candidate) => candidate.token === token,
      )
    ) {
      return;
    }
    if (isTauri()) {
      const generation = ++this.loadGeneration;
      const fallback = $state.snapshot(this.snapshot);
      this.selectedToken = token;
      void selectSource(token)
        .then((snapshot) => {
          if (generation === this.loadGeneration) {
            this.reconcile(snapshot);
          }
        })
        .catch((error: unknown) =>
          this.reconcileCommandFailure(generation, fallback, error),
        );
      return;
    }
    this.selectedToken = token;
  }

  async runDraftCommand(
    action: () => Promise<AppSnapshot>,
    optimistic: (snapshot: AppSnapshot) => AppSnapshot,
  ): Promise<void> {
    const generation = ++this.loadGeneration;
    const fallback = $state.snapshot(this.snapshot);
    this.reconcile(optimistic(fallback));
    try {
      const snapshot = await action();
      if (generation !== this.loadGeneration) {
        return;
      }
      this.reconcile(snapshot);
    } catch (error) {
      await this.reconcileCommandFailure(
        generation,
        fallback,
        error,
        'preferencesWarning',
      );
    }
  }

  async resolveOutcome(
    noticeId: number,
    action?: OutcomeActionId,
  ): Promise<void> {
    const outcome = this.outcome;
    if (!outcome || outcome.id !== noticeId) {
      return;
    }
    if (this.localOutcome) {
      this.localOutcome = null;
      this.dismissedOutcomeId = noticeId;
      return;
    }
    this.dismissedOutcomeId = noticeId;
    if (!isTauri()) {
      this.reconcile({ ...this.backendSnapshot, pendingOutcome: null });
      return;
    }

    const generation = ++this.loadGeneration;
    let actionError: unknown;
    try {
      if (action) {
        try {
          const snapshot = await this.requestOutcomeAction(action);
          if (generation !== this.loadGeneration) {
            return;
          }
          this.reconcile(snapshot);
        } catch (error) {
          actionError = error;
        }
      }
      try {
        const snapshot = await requestAcknowledgeOutcome(noticeId);
        if (generation === this.loadGeneration) {
          this.reconcile(snapshot);
        }
      } catch (error) {
        if (!actionError) {
          actionError = error;
        }
      }
      if (generation === this.loadGeneration && actionError) {
        this.localOutcome = {
          id: noticeId,
          severity: 'error',
          operation: outcome.operation,
          title: copy.outcome.actionErrorTitle,
          message: safeErrorMessage(actionError),
          paths: [],
          actions: [],
        };
      }
    } catch {
      // The action and acknowledgement requests are handled independently.
    }
  }

  private async requestOutcomeAction(
    action: OutcomeActionId,
  ): Promise<AppSnapshot> {
    switch (action) {
      case 'openSessionFolder':
        return openSessionFolder();
      case 'openReport':
        return openReport();
      case 'openReportFolder':
        return openReportFolder();
      case 'openDerivedFolder':
        return openDerivedFolder();
      case 'openCollectionWorkingFolder':
        return openCollectionWorkingFolder();
      case 'openReportWorkingFolder':
        return openReportWorkingFolder();
      case 'openCombineWorkingFolder':
        return openCombineWorkingFolder();
    }
  }

  private async runFolderCommand(
    action: () => Promise<AppSnapshot>,
    operation: OutcomeNotice['operation'],
  ): Promise<void> {
    const generation = ++this.loadGeneration;
    try {
      const snapshot = await action();
      if (generation === this.loadGeneration) {
        this.reconcile(snapshot);
      }
    } catch (error) {
      if (generation !== this.loadGeneration) {
        return;
      }
      this.localOutcome = {
        id: -generation,
        severity: 'error',
        operation,
        title: copy.outcome.actionErrorTitle,
        message: safeErrorMessage(error),
        paths: [],
        actions: [],
      };
    }
  }

  setSampleBits(bits: number): void {
    if (!Number.isInteger(bits) || bits <= 0 || bits % 8 !== 0) {
      this.backendSnapshot = {
        ...this.backendSnapshot,
        preferencesWarning:
          'Sample size must be a positive multiple of 8 bits.',
      };
      return;
    }
    if (isTauri()) {
      void this.runDraftCommand(
        () => setSampleBits(bits),
        (snapshot) => ({
          ...snapshot,
          collection: { ...snapshot.collection, sampleBits: bits },
          preferencesWarning: null,
        }),
      );
      return;
    }
    this.backendSnapshot = {
      ...this.backendSnapshot,
      collection: { ...this.backendSnapshot.collection, sampleBits: bits },
      preferencesWarning: null,
    };
  }

  setIntervalSeconds(seconds: number): void {
    if (!Number.isInteger(seconds) || seconds < 1) {
      this.backendSnapshot = {
        ...this.backendSnapshot,
        preferencesWarning: 'Sample interval must be at least one second.',
      };
      return;
    }
    if (isTauri()) {
      void this.runDraftCommand(
        () => setIntervalSeconds(seconds),
        (snapshot) => ({
          ...snapshot,
          collection: { ...snapshot.collection, intervalSeconds: seconds },
          preferencesWarning: null,
        }),
      );
      return;
    }
    this.backendSnapshot = {
      ...this.backendSnapshot,
      collection: {
        ...this.backendSnapshot.collection,
        intervalSeconds: seconds,
      },
      preferencesWarning: null,
    };
  }

  setFold(fold: number): void {
    if (!Number.isInteger(fold) || fold < 0 || fold > 4) {
      this.backendSnapshot = {
        ...this.backendSnapshot,
        preferencesWarning: 'Fold must be between 0 and 4.',
      };
      return;
    }
    if (isTauri()) {
      void this.runDraftCommand(
        () => setFold(fold),
        (snapshot) => ({
          ...snapshot,
          collection: { ...snapshot.collection, fold },
          preferencesWarning: null,
        }),
      );
      return;
    }
    this.backendSnapshot = {
      ...this.backendSnapshot,
      collection: { ...this.backendSnapshot.collection, fold },
      preferencesWarning: null,
    };
  }

  setTheme(theme: ThemePreference): void {
    this.theme = theme;
    if (isTauri()) {
      void this.runDraftCommand(
        () => setTheme(theme),
        (snapshot) => ({ ...snapshot, theme, preferencesWarning: null }),
      );
      return;
    }
    this.backendSnapshot = { ...this.backendSnapshot, theme };
  }

  chooseOutputFolder(): void {
    if (isTauri()) {
      void this.runDraftCommand(
        () => chooseOutputFolder(),
        (snapshot) => snapshot,
      );
      return;
    }
    if (!import.meta.env.DEV) {
      return;
    }
    this.backendSnapshot = {
      ...this.backendSnapshot,
      collection: {
        ...this.backendSnapshot.collection,
        outputRootLabel:
          this.backendSnapshot.collection.outputRootLabel ?? 'Chosen folder',
      },
      preferencesWarning: null,
    };
  }

  acceptCollectionEvent(event: CollectionEvent): void {
    const collection = this.backendSnapshot.collection;
    if (collection.state !== 'collecting' && collection.state !== 'stopping') {
      return;
    }
    if (event.sessionId !== collection.sessionId) {
      if (
        collection.sessionId !== null ||
        (collection.state !== 'collecting' && collection.state !== 'stopping')
      ) {
        return;
      }
    }
    if (event.sequence <= collection.lastEventSequence) {
      return;
    }
    const next = {
      ...collection,
      sessionId: event.sessionId,
      lastEventSequence: event.sequence,
    };
    switch (event.kind) {
      case 'sessionStarted':
        next.sessionStem = event.stem;
        this.chartSeries.clear();
        this.chartVersion += 1;
        break;
      case 'sampleCommitted':
        next.sampleCount = event.sampleCount;
        next.elapsedLabel = event.elapsedLabel;
        next.onesProportionLabel = event.onesProportionLabel;
        next.cumulativeZLabel = event.cumulativeZLabel;
        this.chartSeries.append(event.sampleIndex, event.cumulativeZ);
        this.chartVersion += 1;
        break;
      case 'timingOverrun':
        next.overrunCount = event.overrunCount;
        break;
      case 'cleanStop':
        next.state = 'completed';
        next.statusLabel = 'Completed';
        next.sampleCount = event.sampleCount;
        next.overrunCount = event.overrunCount;
        next.errorCode = null;
        next.errorMessage = null;
        next.errorRecovery = null;
        break;
      case 'terminalFailure':
        next.state = 'failed';
        next.statusLabel = 'Failed';
        next.errorCode = event.code;
        next.errorMessage = event.message;
        next.errorRecovery = event.recovery ?? null;
        break;
    }
    this.reconcile({ ...this.backendSnapshot, collection: next });
  }

  startCollection(): void {
    if (isTauri()) {
      const generation = ++this.loadGeneration;
      const channelGeneration = ++this.collectionChannelGeneration;
      const fallback = $state.snapshot(this.snapshot);
      this.chartSeries.clear();
      this.chartVersion += 1;
      this.reconcile({
        ...fallback,
        collection: {
          ...fallback.collection,
          state: 'collecting',
          statusLabel: 'Collecting',
          errorCode: null,
          errorMessage: null,
          errorRecovery: null,
        },
      });
      void startCollection((event) => {
        if (channelGeneration === this.collectionChannelGeneration) {
          this.acceptCollectionEvent(event);
        }
      })
        .then((snapshot) => {
          if (generation !== this.loadGeneration) {
            return;
          }
          const current = this.backendSnapshot.collection;
          if (
            snapshot.collection.sessionId === current.sessionId &&
            snapshot.collection.lastEventSequence < current.lastEventSequence
          ) {
            return;
          }
          this.reconcile(snapshot);
        })
        .catch((error: unknown) =>
          this.reconcileCommandFailure(generation, fallback, error),
        );
      return;
    }
    if (this.snapshot.collection.state !== 'ready') {
      return;
    }
    this.chartSeries.clear();
    this.chartVersion += 1;
    this.reconcile({
      ...this.snapshot,
      collection: {
        ...this.snapshot.collection,
        state: 'collecting',
        statusLabel: 'Collecting',
        sessionId: 's1',
        lastEventSequence: 0,
        errorCode: null,
        errorMessage: null,
        errorRecovery: null,
      },
    });
  }

  stopCollection(): void {
    if (isTauri()) {
      const generation = ++this.loadGeneration;
      const fallback = $state.snapshot(this.snapshot);
      this.reconcile({
        ...fallback,
        collection: {
          ...fallback.collection,
          state: 'stopping',
          statusLabel: 'Stopping',
        },
      });
      void stopCollection()
        .then((snapshot) => {
          if (generation !== this.loadGeneration) {
            return;
          }
          const current = this.backendSnapshot.collection;
          if (
            snapshot.collection.sessionId === current.sessionId &&
            snapshot.collection.lastEventSequence < current.lastEventSequence
          ) {
            return;
          }
          this.reconcile(snapshot);
        })
        .catch((error: unknown) =>
          this.reconcileCommandFailure(generation, fallback, error),
        );
      return;
    }
    if (
      this.snapshot.collection.state !== 'collecting' &&
      this.snapshot.collection.state !== 'stopping'
    ) {
      return;
    }
    this.reconcile({
      ...this.snapshot,
      collection: {
        ...this.snapshot.collection,
        state: 'completed',
        statusLabel: 'Completed',
      },
    });
  }

  startAnotherSession(): void {
    if (isTauri()) {
      const generation = ++this.loadGeneration;
      ++this.collectionChannelGeneration;
      const fallback = $state.snapshot(this.snapshot);
      void startAnotherSession()
        .then((snapshot) => {
          if (generation === this.loadGeneration) {
            this.reconcile(snapshot);
            this.chartSeries.clear();
            this.chartVersion += 1;
          }
        })
        .catch((error: unknown) =>
          this.reconcileCommandFailure(generation, fallback, error),
        );
      return;
    }
    if (
      this.snapshot.collection.state !== 'completed' &&
      this.snapshot.collection.state !== 'failed'
    ) {
      return;
    }
    const ready = Boolean(this.snapshot.collection.outputRootLabel);
    this.chartSeries.clear();
    this.chartVersion += 1;
    this.reconcile({
      ...this.snapshot,
      collection: {
        ...this.snapshot.collection,
        state: ready ? 'ready' : 'idle',
        statusLabel: ready ? 'Ready' : 'Idle',
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
        errorRecovery: null,
      },
    });
  }

  openSessionFolder(): void {
    if (isTauri()) {
      void this.runFolderCommand(() => openSessionFolder(), 'collection');
    }
  }

  openCollectionWorkingFolder(): void {
    if (isTauri()) {
      void this.runFolderCommand(
        () => openCollectionWorkingFolder(),
        'collection',
      );
    }
  }

  chooseReportInput(): void {
    if (isTauri()) {
      void this.runDraftCommand(
        () => chooseReportInput(),
        (snapshot) => snapshot,
      );
    }
  }

  generateReport(): void {
    const preview = this.snapshot.reports.preview;
    if (!preview) {
      return;
    }
    if (preview.conflict) {
      this.replaceDialogOpen = true;
      return;
    }
    if (isTauri()) {
      void this.runDraftCommand(
        () => generateReport(),
        (snapshot) => snapshot,
      );
      return;
    }
    this.replaceDialogOpen = false;
    this.reconcile({
      ...this.snapshot,
      reports: {
        preview: { ...preview, conflict: true },
        reportReady: true,
      },
    });
  }

  confirmReplaceReport(): void {
    this.replaceDialogOpen = false;
    if (isTauri()) {
      void this.runDraftCommand(
        () => replaceReport(),
        (snapshot) => snapshot,
      );
      return;
    }
    const preview = this.snapshot.reports.preview;
    if (!preview) {
      return;
    }
    this.reconcile({
      ...this.snapshot,
      reports: {
        preview: { ...preview, conflict: true },
        reportReady: true,
      },
    });
  }

  cancelReplaceReport(): void {
    this.replaceDialogOpen = false;
  }

  openReport(): void {
    if (isTauri()) {
      void this.runFolderCommand(() => openReport(), 'report');
    }
  }

  openReportFolder(): void {
    if (isTauri()) {
      void this.runFolderCommand(() => openReportFolder(), 'report');
    }
  }

  openReportWorkingFolder(): void {
    if (isTauri()) {
      void this.runFolderCommand(() => openReportWorkingFolder(), 'report');
    }
  }

  chooseCsvInputs(): void {
    if (isTauri()) {
      void this.runDraftCommand(
        () => chooseCsvInputs(),
        (snapshot) => snapshot,
      );
    }
  }

  removeCombineInput(inputId: string): void {
    if (isTauri()) {
      void this.runDraftCommand(
        () => removeCombineInput(inputId),
        (snapshot) => snapshot,
      );
    }
  }

  clearCombineInputs(): void {
    if (isTauri()) {
      void this.runDraftCommand(
        () => clearCombineInputs(),
        (snapshot) => snapshot,
      );
    }
  }

  createDerived(): void {
    if (isTauri()) {
      void this.runDraftCommand(
        () => createDerived(),
        (snapshot) => snapshot,
      );
    }
  }

  generateDerived(): void {
    const preview = this.snapshot.reports.preview;
    if (
      preview?.kindLabel === 'Derived bundle' &&
      preview.conflict &&
      this.snapshot.combine.result
    ) {
      this.replaceDialogOpen = true;
      return;
    }
    if (isTauri()) {
      void this.runDraftCommand(
        () => generateDerived(false),
        (snapshot) => snapshot,
      );
    }
  }

  confirmReplaceDerived(): void {
    this.replaceDialogOpen = false;
    if (isTauri()) {
      void this.runDraftCommand(
        () => generateDerived(true),
        (snapshot) => snapshot,
      );
    }
  }

  openDerivedFolder(): void {
    if (isTauri()) {
      void this.runFolderCommand(() => openDerivedFolder(), 'combine');
    }
  }

  openCombineWorkingFolder(): void {
    if (isTauri()) {
      void this.runFolderCommand(() => openCombineWorkingFolder(), 'combine');
    }
  }

  async refreshSources(): Promise<void> {
    const generation = ++this.loadGeneration;
    const fallback = $state.snapshot(this.snapshot);
    this.backendSnapshot = {
      ...this.backendSnapshot,
      collection: {
        ...this.backendSnapshot.collection,
        state: 'discovering',
        statusLabel: 'Discovering sources',
        candidates: [],
        selectedToken: null,
        familyWarning: null,
      },
    };
    this.selectedToken = null;
    try {
      const snapshot = await refreshSources();
      if (generation !== this.loadGeneration) {
        return;
      }
      this.reconcile(snapshot);
    } catch (error) {
      await this.reconcileCommandFailure(generation, fallback, error);
    }
  }

  reset(): void {
    this.loadGeneration += 1;
    this.destination = 'collect';
    this.theme = 'system';
    this.scenarioId = DEFAULT_SCENARIO;
    this.replaceDialogOpen = false;
    this.closePrompt = 'none';
    this.diagnosticsCopied = false;
    this.localOutcome = null;
    this.dismissedOutcomeId = null;
    this.reconcile(cloneSnapshot(DEFAULT_SCENARIO));
    this.chartSeries.clear();
    this.chartVersion += 1;
  }

  private syncMockChart(collection: CollectionSnapshot): void {
    if (
      collection.sampleCount > 0 &&
      (collection.state === 'collecting' ||
        collection.state === 'stopping' ||
        collection.state === 'completed' ||
        collection.state === 'failed')
    ) {
      const seeded = syntheticCumulativeZ(collection.sampleCount);
      this.chartSeries.replaceAll(seeded.sampleIndex, seeded.cumulativeZ);
    } else {
      this.chartSeries.clear();
    }
    this.chartVersion += 1;
  }
}

export const appState = new AppViewState();

export function resetAppState(): void {
  appState.reset();
}

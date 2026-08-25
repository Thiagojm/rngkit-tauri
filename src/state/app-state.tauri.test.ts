import { afterEach, describe, expect, it, vi } from 'vitest';
import type { AppSnapshot, CollectionEvent } from '../ipc/types';
import { MOCK_SCENARIOS } from './mock-scenarios';

afterEach(() => {
  vi.useRealTimers();
  vi.doUnmock('@tauri-apps/api/core');
  vi.doUnmock('../ipc/client');
  vi.resetModules();
});

describe('AppViewState native collection channel', () => {
  it('automatically discovers once after idle hydration without selecting a source', async () => {
    const idle = structuredClone(MOCK_SCENARIOS.idle);
    const discovered = structuredClone(MOCK_SCENARIOS.idle);
    discovered.collection.candidates = structuredClone(
      MOCK_SCENARIOS.ready.collection.candidates,
    );
    const getAppState = vi
      .fn<() => Promise<AppSnapshot>>()
      .mockResolvedValueOnce(idle)
      .mockResolvedValue(discovered);
    const refreshSources = vi.fn(async () => discovered);

    vi.resetModules();
    vi.doMock('@tauri-apps/api/core', async (importOriginal) => ({
      ...(await importOriginal<typeof import('@tauri-apps/api/core')>()),
      isTauri: () => true,
    }));
    vi.doMock('../ipc/client', async (importOriginal) => ({
      ...(await importOriginal<typeof import('../ipc/client')>()),
      getAppState,
      refreshSources,
    }));
    const { AppViewState } = await import('./app-state.svelte');
    const state = new AppViewState();

    await state.hydrate();
    await vi.waitFor(() => expect(refreshSources).toHaveBeenCalledTimes(1));
    await state.hydrate();

    expect(getAppState).toHaveBeenCalledTimes(2);
    expect(refreshSources).toHaveBeenCalledTimes(1);
    expect(state.snapshot.collection.candidates).toHaveLength(2);
    expect(state.snapshot.collection.selectedToken).toBeNull();
  });

  it('accepts CleanStop after Stop and ignores the older stopping response', async () => {
    let onEvent: ((event: CollectionEvent) => void) | undefined;
    let resolveStop: ((snapshot: AppSnapshot) => void) | undefined;
    const ready = structuredClone(MOCK_SCENARIOS.ready);
    const collecting: AppSnapshot = {
      ...ready,
      collection: {
        ...ready.collection,
        state: 'collecting',
        statusLabel: 'Collecting',
        sessionId: 's1',
        lastEventSequence: 0,
      },
    };
    const stopping: AppSnapshot = {
      ...collecting,
      collection: {
        ...collecting.collection,
        state: 'stopping',
        statusLabel: 'Stopping',
      },
    };
    const completed: AppSnapshot = {
      ...stopping,
      collection: {
        ...stopping.collection,
        state: 'completed',
        statusLabel: 'Completed',
        sampleCount: 3,
        lastEventSequence: 1,
      },
      pendingOutcome: {
        id: 1,
        severity: 'success',
        operation: 'collection',
        title: 'Collection saved',
        message: 'The collection artifacts were saved.',
        paths: [
          {
            label: 'Session folder',
            path: 'C:\\Users\\tester\\Documents\\rngkit\\session',
          },
        ],
        actions: ['openSessionFolder'],
      },
    };
    const getAppState = vi.fn(async () => completed);

    vi.resetModules();
    vi.doMock('@tauri-apps/api/core', async (importOriginal) => ({
      ...(await importOriginal<typeof import('@tauri-apps/api/core')>()),
      isTauri: () => true,
    }));
    vi.doMock('../ipc/client', () => ({
      applyDevScenario: vi.fn(),
      chooseCsvInputs: vi.fn(),
      clearCombineInputs: vi.fn(),
      chooseOutputFolder: vi.fn(),
      chooseReportInput: vi.fn(),
      createDerived: vi.fn(),
      generateDerived: vi.fn(),
      generateReport: vi.fn(),
      getAppState,
      openDerivedFolder: vi.fn(),
      openReport: vi.fn(),
      openReportFolder: vi.fn(),
      openSessionFolder: vi.fn(),
      removeCombineInput: vi.fn(),
      replaceReport: vi.fn(),
      refreshSources: vi.fn(),
      safeErrorMessage: () => 'safe error',
      selectSource: vi.fn(),
      setFold: vi.fn(),
      setIntervalSeconds: vi.fn(),
      setSampleBits: vi.fn(),
      setTheme: vi.fn(),
      startAnotherSession: vi.fn(),
      copyDiagnostics: vi.fn(),
      listenCloseRequested: vi.fn(async () => () => undefined),
      stopAndExit: vi.fn(),
      startCollection: vi.fn(
        async (handler: (event: CollectionEvent) => void) => {
          onEvent = handler;
          return collecting;
        },
      ),
      stopCollection: vi.fn(
        () =>
          new Promise<AppSnapshot>((resolve) => {
            resolveStop = resolve;
          }),
      ),
    }));
    const { AppViewState } = await import('./app-state.svelte');
    const state = new AppViewState();
    state.reconcile(ready);

    state.startCollection();
    await vi.waitFor(() => expect(onEvent).toBeTypeOf('function'));
    state.stopCollection();
    onEvent?.({
      kind: 'cleanStop',
      sessionId: 's1',
      sequence: 1,
      sampleCount: 3,
      overrunCount: 0,
    });
    resolveStop?.(stopping);
    await vi.waitFor(() =>
      expect(state.outcome?.title).toBe('Collection saved'),
    );

    expect(getAppState).toHaveBeenCalledTimes(1);
    expect(state.snapshot.collection.state).toBe('completed');
    expect(state.snapshot.collection.sampleCount).toBe(3);
    expect(state.snapshot.collection.lastEventSequence).toBe(1);
  });

  it('keeps reconciling a reloaded active session until it becomes terminal', async () => {
    vi.useFakeTimers();
    const collecting = structuredClone(MOCK_SCENARIOS.collecting);
    const failed = structuredClone(MOCK_SCENARIOS.failed);
    const getAppState = vi
      .fn<() => Promise<AppSnapshot>>()
      .mockResolvedValueOnce(collecting)
      .mockResolvedValue(failed);

    vi.resetModules();
    vi.doMock('@tauri-apps/api/core', async (importOriginal) => ({
      ...(await importOriginal<typeof import('@tauri-apps/api/core')>()),
      isTauri: () => true,
    }));
    vi.doMock('../ipc/client', () => ({
      applyDevScenario: vi.fn(),
      chooseCsvInputs: vi.fn(),
      clearCombineInputs: vi.fn(),
      chooseOutputFolder: vi.fn(),
      chooseReportInput: vi.fn(),
      copyDiagnostics: vi.fn(),
      createDerived: vi.fn(),
      generateDerived: vi.fn(),
      generateReport: vi.fn(),
      getAppState,
      listenCloseRequested: vi.fn(async () => () => undefined),
      openDerivedFolder: vi.fn(),
      openReport: vi.fn(),
      openReportFolder: vi.fn(),
      openSessionFolder: vi.fn(),
      removeCombineInput: vi.fn(),
      replaceReport: vi.fn(),
      refreshSources: vi.fn(),
      safeErrorMessage: () => 'safe error',
      selectSource: vi.fn(),
      setFold: vi.fn(),
      setIntervalSeconds: vi.fn(),
      setSampleBits: vi.fn(),
      setTheme: vi.fn(),
      startAnotherSession: vi.fn(),
      startCollection: vi.fn(),
      stopAndExit: vi.fn(),
      stopCollection: vi.fn(),
    }));
    const { AppViewState } = await import('./app-state.svelte');
    const state = new AppViewState();

    await state.hydrate();
    expect(state.snapshot.collection.state).toBe('collecting');
    await vi.advanceTimersByTimeAsync(250);

    expect(getAppState).toHaveBeenCalledTimes(2);
    expect(state.snapshot.collection.state).toBe('failed');
  });
});

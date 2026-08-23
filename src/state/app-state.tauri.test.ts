import { afterEach, describe, expect, it, vi } from 'vitest';
import type { AppSnapshot, CollectionEvent } from '../ipc/types';
import { MOCK_SCENARIOS } from './mock-scenarios';

afterEach(() => {
  vi.doUnmock('@tauri-apps/api/core');
  vi.doUnmock('../ipc/client');
  vi.resetModules();
});

describe('AppViewState native collection channel', () => {
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

    vi.resetModules();
    vi.doMock('@tauri-apps/api/core', async (importOriginal) => ({
      ...(await importOriginal<typeof import('@tauri-apps/api/core')>()),
      isTauri: () => true,
    }));
    vi.doMock('../ipc/client', () => ({
      applyDevScenario: vi.fn(),
      chooseOutputFolder: vi.fn(),
      getAppState: vi.fn(async () => collecting),
      openSessionFolder: vi.fn(),
      refreshSources: vi.fn(),
      safeErrorMessage: () => 'safe error',
      selectSource: vi.fn(),
      setFold: vi.fn(),
      setIntervalSeconds: vi.fn(),
      setSampleBits: vi.fn(),
      setTheme: vi.fn(),
      startAnotherSession: vi.fn(),
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
      expect(state.snapshot.collection.state).toBe('completed'),
    );

    expect(state.snapshot.collection.sampleCount).toBe(3);
    expect(state.snapshot.collection.lastEventSequence).toBe(1);
  });
});

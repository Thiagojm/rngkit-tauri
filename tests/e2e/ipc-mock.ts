import type { Page } from '@playwright/test';
import type { AppSnapshot, CollectionEvent } from '../../src/ipc/types';
import {
  MOCK_SCENARIOS,
  type ScenarioId,
} from '../../src/state/mock-scenarios';

export const VIEWPORTS = {
  min: { width: 800, height: 600 },
  wide: { width: 1280, height: 800 },
} as const;

type Responses = Record<string, AppSnapshot>;

/**
 * Test-only Tauri IPC mock for the production build. Unknown commands reject
 * so a missing mock fails the test. No native IPC or hardware is reached.
 */
export async function installIpcMock(
  page: Page,
  scenario: ScenarioId,
  responses: Responses = {},
): Promise<void> {
  await page.addInitScript(
    ({ initial, responses }) => {
      const callbacks = new Map<number, (payload: unknown) => void>();
      let nextId = 1;
      let channelId: number | null = null;
      let index = 0;
      const snapshot = initial;
      const w = window as unknown as Record<string, unknown>;
      w.isTauri = true;
      w.__TAURI_INTERNALS__ = {
        transformCallback(cb: (payload: unknown) => void) {
          const id = nextId++;
          callbacks.set(id, cb);
          return id;
        },
        unregisterCallback(id: number) {
          callbacks.delete(id);
        },
        async invoke(cmd: string, args: Record<string, unknown> = {}) {
          if (cmd === 'plugin:event|listen') return nextId++;
          if (cmd === 'plugin:event|unlisten') return null;
          if (
            [
              'get_app_state',
              'refresh_sources',
              'set_sample_bits',
              'set_interval_seconds',
              'set_fold',
            ].includes(cmd)
          )
            return structuredClone(snapshot);
          if (cmd === 'start_collection') {
            channelId = (args.onEvent as { id: number }).id;
            index = 0;
          }
          if (cmd in responses) return structuredClone(responses[cmd]);
          console.error(`ipc-mock: unmocked command ${cmd}`);
          throw new Error(`ipc-mock: unmocked command ${cmd}`);
        },
      };
      w.__ipcMockEmit = (message: unknown) => {
        if (channelId === null) throw new Error('ipc-mock: no channel');
        callbacks.get(channelId)?.({ index: index++, message });
      };
    },
    { initial: MOCK_SCENARIOS[scenario], responses },
  );
}

export async function emit(page: Page, events: CollectionEvent[]) {
  await page.evaluate((events) => {
    const send = (window as unknown as Record<string, (m: unknown) => void>)
      .__ipcMockEmit;
    for (const event of events) send(event);
  }, events);
}

/** A started session followed by `count` committed samples. */
export function sampleEvents(count: number, sessionId = 's1') {
  const events: CollectionEvent[] = [
    { kind: 'sessionStarted', sessionId, sequence: 1, stem: 'mock_session' },
  ];
  let z = 0;
  for (let i = 1; i <= count; i += 1) {
    z += Math.sin(i / 3) * 0.4;
    events.push({
      kind: 'sampleCommitted',
      sessionId,
      sequence: i + 1,
      sampleIndex: i,
      sampleCount: i,
      elapsedLabel: '00:00:01',
      onesProportionLabel: '0.5000',
      cumulativeZ: z,
      cumulativeZLabel: z.toFixed(2),
    });
  }
  return events;
}

export async function box(page: Page, selector: string) {
  const rect = await page.locator(selector).first().boundingBox();
  if (!rect) throw new Error(`no box for ${selector}`);
  return rect;
}

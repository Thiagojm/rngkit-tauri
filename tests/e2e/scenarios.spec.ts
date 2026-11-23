import { expect, test } from '@playwright/test';
import { copy } from '../../src/copy';
import { MOCK_SCENARIOS } from '../../src/state/mock-scenarios';
import { VIEWPORTS, emit, installIpcMock, sampleEvents } from './ipc-mock';

const started = structuredClone(MOCK_SCENARIOS.collecting);
started.collection.lastEventSequence = 0;

test('mocked IPC drives a collecting session and a populated chart', async ({
  page,
}) => {
  await installIpcMock(page, 'ready', { start_collection: started });
  await page.setViewportSize(VIEWPORTS.wide);
  await page.goto('/');

  await page.getByRole('button', { name: copy.start }).click();
  await expect(page.getByText(`${copy.status}: Collecting`)).toBeVisible();
  await emit(page, sampleEvents(40));

  await expect(
    page.getByText(`${copy.chart.retainedPoints}: 40`, { exact: false }),
  ).toBeVisible();
  await expect(page.getByText(copy.chart.empty)).toHaveCount(0);
  await expect(
    page.getByRole('img', { name: copy.chart.caption }),
  ).toBeVisible();
  await expect(page.getByTestId('dev-scenario-switch')).toHaveCount(0);
});

import { expect, test } from '@playwright/test';
import { copy } from '../../src/copy';
import { MOCK_SCENARIOS } from '../../src/state/mock-scenarios';
import { VIEWPORTS, box, emit, installIpcMock, sampleEvents } from './ipc-mock';

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

for (const [size, stacked] of [
  ['min', true],
  ['wide', false],
] as const) {
  test(`Collect layout at ${size}`, async ({ page }) => {
    await installIpcMock(page, 'ready', { start_collection: started });
    await page.setViewportSize(VIEWPORTS[size]);
    await page.goto('/');
    await page.getByRole('button', { name: copy.start }).click();
    await emit(page, sampleEvents(40));
    await expect(
      page.getByText(`${copy.chart.retainedPoints}: 40`),
    ).toBeVisible();

    const session = await box(
      page,
      'section[aria-labelledby="collect-config"]',
    );
    const monitor = await box(
      page,
      'section[aria-labelledby="collect-monitor"]',
    );
    expect(monitor.y > session.y + session.height - 1).toBe(stacked);
    const plot = await box(page, '.live-z-chart .u-over');
    if (stacked) {
      expect(plot.width).toBeGreaterThanOrEqual(480);
      expect((await box(page, '.live-z-chart')).height).toBeGreaterThanOrEqual(
        19 * 16,
      );
    }
    for (const value of await page
      .locator('[data-testid="metric-value"]')
      .all()) {
      const lines = await value.evaluate(
        (el) =>
          el.getBoundingClientRect().height /
          parseFloat(getComputedStyle(el).lineHeight),
      );
      expect(lines).toBeLessThan(1.5);
    }
    const overflow = await page.evaluate(() => {
      const main = document.getElementById('main-content')!;
      return main.scrollWidth - main.clientWidth;
    });
    expect(overflow).toBeLessThanOrEqual(0);
  });
}

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

test.use({ launchOptions: { ignoreDefaultArgs: ['--hide-scrollbars'] } });

test.describe('Collect layout with visible scrollbars', () => {
  for (const [size, stacked] of [
    ['min', true],
    ['wide', false],
  ] as const) {
    test(`Collect layout at ${size}`, async ({ page }) => {
      await installIpcMock(page, 'ready', { start_collection: started });
      if (stacked) {
        await page.addInitScript(() => {
          const fillText = CanvasRenderingContext2D.prototype.fillText;
          CanvasRenderingContext2D.prototype.fillText = function (...args) {
            const [text, x] = args;
            if (text === '1000000') {
              const right = x + this.measureText(text).actualBoundingBoxRight;
              this.canvas.dataset.lastTickFits = String(
                right <= this.canvas.width,
              );
            }
            fillText.apply(this, args);
          };
        });
      }

      await page.setViewportSize(VIEWPORTS[size]);
      await page.goto('/');
      if (stacked) {
        await page.addStyleTag({
          content: 'html::-webkit-scrollbar { width: 16px; }',
        });
        const gutter = () =>
          page.evaluate(
            () => innerWidth - document.documentElement.clientWidth,
          );
        await expect.poll(gutter).toBe(16);
      }

      await page.getByRole('button', { name: copy.start }).click();
      const events = sampleEvents(40).map((event) =>
        event.kind === 'sampleCommitted'
          ? {
              ...event,
              sampleIndex: event.sampleIndex + 999960,
              sampleCount: event.sampleCount + 999960,
              elapsedLabel: '277:46:40',
            }
          : event,
      );
      await emit(page, events);
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
      if (stacked) {
        await expect
          .poll(async () => (await box(page, '.live-z-chart .u-over')).width)
          .toBeGreaterThanOrEqual(480);
        expect(
          (await box(page, '.live-z-chart')).height,
        ).toBeGreaterThanOrEqual(19 * 16);
        await expect(page.locator('.live-z-chart canvas')).toHaveAttribute(
          'data-last-tick-fits',
          'true',
        );
      }
      const metrics = page.getByTestId('metric-value');
      await expect(metrics).toHaveCount(5);
      await expect(metrics.first()).toHaveText('1000000');
      for (const value of await metrics.all()) {
        const layout = await value.evaluate((el) => ({
          lines:
            el.getBoundingClientRect().height /
            parseFloat(getComputedStyle(el).lineHeight),
          overflow: el.scrollWidth - el.clientWidth,
        }));
        expect(layout.lines).toBeLessThan(1.5);
        expect(layout.overflow).toBeLessThanOrEqual(0);
      }

      const overflow = await page.evaluate(() => {
        const main = document.getElementById('main-content')!;
        return main.scrollWidth - main.clientWidth;
      });
      expect(overflow).toBeLessThanOrEqual(0);
    });
  }
});

test('drag-zoom pauses following and double-click resumes it', async ({
  page,
}) => {
  await installIpcMock(page, 'ready', { start_collection: started });
  await page.setViewportSize(VIEWPORTS.wide);
  await page.goto('/');
  await page.getByRole('button', { name: copy.start }).click();
  await emit(page, sampleEvents(40));
  const status = page.getByTestId('chart-point-count');
  await expect(status).toContainText(copy.chart.following);

  const over = await box(page, '.live-z-chart .u-over');
  await page.mouse.move(over.x + 60, over.y + 40);
  await page.mouse.down();
  await page.mouse.move(over.x + 200, over.y + 120, { steps: 5 });
  await page.mouse.up();
  await expect(status).toContainText(copy.chart.paused);

  await page.mouse.dblclick(over.x + 100, over.y + 60);
  await expect(status).toContainText(copy.chart.following);
});

for (const [size, layout] of [
  ['min', 'list'],
  ['wide', 'table'],
] as const) {
  test(`Combine inputs render as a ${layout} at ${size}`, async ({ page }) => {
    await installIpcMock(page, 'combineCompatible');
    await page.setViewportSize(VIEWPORTS[size]);
    await page.goto('/');
    await page.getByRole('button', { name: copy.destinations.combine }).click();
    const name = 'CSV inputs in derived chronological order';
    await expect(page.getByRole(layout, { name })).toBeVisible();
    await expect(
      page.getByRole(layout === 'list' ? 'table' : 'list', { name }),
    ).toHaveCount(0);
    await expect(
      page.getByRole('button', {
        name: 'Remove 20260101T010000_bitb_s8_i1.csv #1',
      }),
    ).toBeVisible();
    const overflow = await page.evaluate(() => {
      const main = document.getElementById('main-content')!;
      return main.scrollWidth - main.clientWidth;
    });
    expect(overflow).toBeLessThanOrEqual(0);
  });
}

test('Combine keeps Remove focused and fits fractional timestamps across layouts', async ({
  page,
}) => {
  const snapshot = structuredClone(MOCK_SCENARIOS.combineCompatible);
  for (const row of snapshot.combine.inputs) {
    row.firstTimestamp = row.firstTimestamp.replace('Z', '.123456789Z');
    row.lastTimestamp = row.lastTimestamp.replace('Z', '.987654321Z');
    row.rows = 524288;
  }

  await installIpcMock(page, snapshot);
  await page.setViewportSize({ width: 1120, height: 750 });
  await page.goto('/');
  await page.getByRole('button', { name: copy.destinations.combine }).click();
  const name = 'CSV inputs in derived chronological order';
  const cards = page.getByRole('list', { name });
  const table = page.getByRole('table', { name });
  const remove = page.getByRole('button', {
    name: 'Remove 20260101T010000_bitb_s8_i1.csv #1',
  });

  await expect(cards).toBeVisible();
  await remove.focus();

  await page.setViewportSize(VIEWPORTS.wide);
  await expect(table).toBeVisible();
  await expect(remove).toBeFocused();
  await expect(
    table.getByText(snapshot.combine.inputs[0].firstTimestamp),
  ).toBeVisible();
  const overflow = await page
    .getByTestId('combine-inputs')
    .evaluate((el) => el.scrollWidth - el.clientWidth);
  expect(overflow).toBeLessThanOrEqual(0);
  await expect(remove).toBeInViewport();

  await page.setViewportSize({ width: 1120, height: 750 });
  await expect(cards).toBeVisible();
  await expect(remove).toBeFocused();
});

import { expect, test, type Page } from '@playwright/test';
import { copy } from '../../src/copy';
import {
  MOCK_SCENARIOS,
  type ScenarioId,
} from '../../src/state/mock-scenarios';
import { VIEWPORTS, box, installIpcMock } from './ipc-mock';

const noOverflow = (page: Page) =>
  page.evaluate(() => {
    const main = document.getElementById('main-content')!;
    return main.scrollWidth - main.clientWidth;
  });

async function open(
  page: Page,
  scenario: ScenarioId,
  size: keyof typeof VIEWPORTS,
  scheme: 'light' | 'dark',
  responses = {},
) {
  await installIpcMock(page, scenario, responses);
  await page.emulateMedia({ colorScheme: scheme });
  await page.setViewportSize(VIEWPORTS[size]);
  await page.goto('/');
}

for (const size of ['min', 'wide'] as const) {
  for (const scheme of ['light', 'dark'] as const) {
    test(`Collect source actions stay put across states (${size}, ${scheme})`, async ({
      page,
    }) => {
      const positions: string[] = [];
      for (const scenario of [
        'ready',
        'collecting',
        'stopping',
        'completed',
      ] as const) {
        await open(page, scenario, size, scheme);
        await expect(
          page.getByText(
            `${copy.status}: ${MOCK_SCENARIOS[scenario].collection.statusLabel}`,
          ),
        ).toBeVisible();
        const refresh = await page
          .getByRole('button', { name: copy.refreshSources })
          .boundingBox();
        const setup = await page
          .getByRole('button', { name: copy.deviceSetup })
          .boundingBox();
        positions.push(JSON.stringify([refresh, setup]));
        expect(await noOverflow(page)).toBeLessThanOrEqual(0);
        if (size === 'wide' && scenario === 'collecting') {
          const monitor = await box(
            page,
            'section[aria-labelledby="collect-monitor"]',
          );
          expect(monitor.y + monitor.height).toBeLessThanOrEqual(
            VIEWPORTS.wide.height,
          );
        }
      }
      expect(new Set(positions).size).toBe(1);
    });

    test(`Reports preview names and replace flow (${size}, ${scheme})`, async ({
      page,
    }) => {
      const replaced = structuredClone(MOCK_SCENARIOS.reportsConflict);
      await open(page, 'reportsConflict', size, scheme, {
        replace_report: replaced,
      });
      await page
        .getByRole('button', { name: copy.destinations.reports })
        .click();
      const preview = MOCK_SCENARIOS.reportsConflict.reports.preview!;
      await expect(page.getByText(preview.inputBasename)).toBeVisible();
      await expect(page.getByText(preview.reportBasename)).toBeVisible();
      await page.getByRole('button', { name: copy.generateReport }).click();
      const dialog = page.getByRole('dialog');
      await expect(dialog.getByText(copy.replaceBody)).toBeVisible();
      await dialog.getByRole('button', { name: copy.cancel }).click();
      await expect(dialog).toHaveCount(0);
      await page.getByRole('button', { name: copy.generateReport }).click();
      await page
        .getByRole('dialog')
        .getByRole('button', { name: copy.replace })
        .click();
      await expect(page.getByRole('dialog')).toHaveCount(0);
      expect(await noOverflow(page)).toBeLessThanOrEqual(0);
    });

    test(`Combine states (${size}, ${scheme})`, async ({ page }) => {
      await open(page, 'combineIncompatible', size, scheme);
      await page
        .getByRole('button', { name: copy.destinations.combine })
        .click();
      await expect(
        page.getByRole('button', { name: copy.createDerived }),
      ).toBeDisabled();
      await expect(page.getByRole('status')).toBeVisible();
      await expect(
        page.getByRole('button', { name: copy.generateXlsx }),
      ).toHaveCount(0);
      const layout = size === 'min' ? 'list' : 'table';
      await expect(
        page.getByRole(layout, {
          name: 'CSV inputs in derived chronological order',
        }),
      ).toBeVisible();
      await expect(
        page.getByRole('button', { name: /^Remove .+ #1$/ }),
      ).toBeVisible();
      expect(await noOverflow(page)).toBeLessThanOrEqual(0);
    });
  }
}

test('Combine empty state hides the list and result actions', async ({
  page,
}) => {
  await open(page, 'ready', 'min', 'light');
  await page.getByRole('button', { name: copy.destinations.combine }).click();
  await expect(page.getByText(copy.noCombineInputs)).toBeVisible();
  await expect(
    page.getByRole('button', { name: copy.createDerived }),
  ).toBeDisabled();
  await expect(
    page.getByRole('button', { name: copy.openCombinedFolder }),
  ).toHaveCount(0);
});

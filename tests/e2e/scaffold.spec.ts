import { expect, test, type Page } from '@playwright/test';
import { copy } from '../../src/copy';

const LIGHT_SURFACE = 'rgb(244, 247, 251)';
const DARK_SURFACE = 'rgb(16, 24, 38)';

async function bodyBackground(page: Page) {
  return page.evaluate(() => getComputedStyle(document.body).backgroundColor);
}

async function hasHorizontalPageScroll(page: Page) {
  return page.evaluate(
    () =>
      document.documentElement.scrollWidth >
      document.documentElement.clientWidth + 1,
  );
}

test('renders the four-destination shell without hardware or mock-scenario controls', async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/');

  await expect(
    page.getByText(copy.product, { exact: true }).first(),
  ).toBeVisible();
  await expect(
    page.getByRole('heading', { name: copy.destinations.collect }),
  ).toBeVisible();
  await expect(
    page.getByRole('navigation', { name: copy.primaryNav }),
  ).toBeVisible();
  await expect(page.getByText(copy.statsWarning)).toBeVisible();
  await expect(page.getByText(copy.chart.empty)).toBeVisible();
  await expect(
    page.getByRole('button', { name: copy.chart.resetView }),
  ).toBeDisabled();
  await expect(page.getByText(`${copy.status}: Idle`)).toBeVisible();
  await expect(
    page.getByRole('button', { name: copy.refreshSources }),
  ).toBeVisible();
  await expect(page.getByText(copy.noSources)).toBeVisible();
  await expect(
    page.getByRole('button', { name: copy.chooseFolder }),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: copy.start })).toBeDisabled();
  await page.getByRole('button', { name: copy.refreshSources }).click();
  await expect(page.getByText(copy.noSources)).toBeVisible();
  await expect(page.getByRole('radio')).toHaveCount(0);
  await expect(page.getByTestId('dev-scenario-switch')).toHaveCount(0);
  await expect(page.getByText('Development scenario')).toHaveCount(0);
  const scriptSrc = await page
    .locator('script[src*="assets/"]')
    .first()
    .getAttribute('src');
  expect(scriptSrc).toBeTruthy();
  const js = await (await page.request.get(scriptSrc!)).text();
  expect(js).not.toContain('apply_dev_scenario');
  expect(js).toContain('choose_output_folder');
  expect(js).toContain('set_sample_bits');
  expect(js).toContain('set_theme');
  expect(js).toContain('start_collection');
  expect(js).toContain('stop_collection');
  expect(js).toContain('open_session_folder');
  expect(js).not.toContain('apply_dev_scenario');
  await expect.poll(() => bodyBackground(page)).toBe(LIGHT_SURFACE);

  await page.getByRole('button', { name: copy.destinations.reports }).click();
  await expect(
    page.getByRole('heading', { name: copy.destinations.reports }),
  ).toBeVisible();

  await page.getByRole('button', { name: copy.destinations.combine }).click();
  await expect(
    page.getByRole('heading', { name: copy.destinations.combine }),
  ).toBeVisible();

  await page.getByRole('button', { name: copy.destinations.help }).click();
  await expect(
    page.getByRole('heading', { name: copy.destinations.help }),
  ).toBeVisible();
  await expect(page.getByText(copy.fold.raw)).toBeVisible();
  await expect(page.getByText(copy.statsWarning)).toBeVisible();

  await page.getByLabel(copy.theme.legend).selectOption('dark');
  await expect.poll(() => bodyBackground(page)).toBe(DARK_SURFACE);

  await page.getByLabel(copy.theme.legend).selectOption('light');
  await expect.poll(() => bodyBackground(page)).toBe(LIGHT_SURFACE);
});

test('stacks the Collect layout at the minimum window without page scroll', async ({
  page,
}) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/');

  await expect(
    page.getByRole('heading', { name: copy.destinations.collect }),
  ).toBeVisible();
  expect(await hasHorizontalPageScroll(page)).toBe(false);

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole('navigation', { name: copy.primaryNav }),
  ).toBeVisible();
  expect(await hasHorizontalPageScroll(page)).toBe(false);
});

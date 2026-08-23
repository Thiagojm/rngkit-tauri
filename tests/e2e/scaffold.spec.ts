import { expect, test } from '@playwright/test';

const LIGHT_SURFACE = 'rgb(244, 247, 251)';
const DARK_SURFACE = 'rgb(16, 24, 38)';

async function bodyBackground(page: import('@playwright/test').Page) {
  return page.evaluate(() => getComputedStyle(document.body).backgroundColor);
}

test('renders the scaffold with conditional system theme tokens', async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'RngKit' })).toBeVisible();
  await expect.poll(() => bodyBackground(page)).toBe(LIGHT_SURFACE);

  await page.emulateMedia({ colorScheme: 'dark' });
  await expect.poll(() => bodyBackground(page)).toBe(DARK_SURFACE);
});

import { expect, test } from '@playwright/test';
import { copy } from '../../src/copy';

test('keyboard users can skip to main and reach every destination', async ({
  page,
}) => {
  await page.goto('/');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: copy.skipToMain })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('#main-content')).toBeFocused();

  const nav = page.getByRole('navigation', { name: copy.primaryNav });
  await nav.getByRole('button', { name: copy.destinations.help }).focus();
  await page.keyboard.press('Enter');
  await expect(
    page.getByRole('heading', { name: copy.destinations.help }),
  ).toBeVisible();
  await expect(
    page.getByRole('heading', { name: 'Accessibility' }),
  ).toBeVisible();

  await nav
    .getByRole('button', { name: copy.destinations.reports })
    .press('Enter');
  await expect(
    page.getByRole('heading', { name: copy.destinations.reports }),
  ).toBeVisible();
  await nav
    .getByRole('button', { name: copy.destinations.combine })
    .press('Enter');
  await expect(
    page.getByRole('heading', { name: copy.destinations.combine }),
  ).toBeVisible();
  await nav
    .getByRole('button', { name: copy.destinations.collect })
    .press('Enter');
  await expect(
    page.getByRole('heading', { name: copy.destinations.collect }),
  ).toBeVisible();
  await expect(page.getByText(`${copy.status}: Idle`)).toBeVisible();
});

test('reduced motion, contrast, and scaled minimum window stay usable', async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: 'reduce', colorScheme: 'light' });
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/');

  const motion = await page.evaluate(() => {
    const styles = getComputedStyle(document.body);
    const duration = styles.animationDuration || '0s';
    const seconds = duration.endsWith('ms')
      ? Number.parseFloat(duration) / 1000
      : Number.parseFloat(duration);
    return {
      seconds: Number.isFinite(seconds) ? seconds : 0,
      scrollBehavior: getComputedStyle(document.documentElement).scrollBehavior,
    };
  });
  expect(motion.seconds).toBeLessThan(0.05);
  expect(motion.scrollBehavior === 'auto' || motion.scrollBehavior === '').toBe(
    true,
  );

  const colors = await page.evaluate(() => {
    const body = getComputedStyle(document.body);
    const parse = (value: string) => {
      const match = value.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
      if (!match) {
        return { r: 0, g: 0, b: 0 };
      }
      return {
        r: Number(match[1]),
        g: Number(match[2]),
        b: Number(match[3]),
      };
    };
    const channel = (value: number) => {
      const scaled = value / 255;
      return scaled <= 0.03928
        ? scaled / 12.92
        : ((scaled + 0.055) / 1.055) ** 2.4;
    };
    const luminance = (rgb: { r: number; g: number; b: number }) =>
      0.2126 * channel(rgb.r) +
      0.7152 * channel(rgb.g) +
      0.0722 * channel(rgb.b);
    const fg = parse(body.color);
    const bg = parse(body.backgroundColor);
    const lighter = Math.max(luminance(fg), luminance(bg));
    const darker = Math.min(luminance(fg), luminance(bg));
    return (lighter + 0.05) / (darker + 0.05);
  });
  expect(colors).toBeGreaterThanOrEqual(4.5);

  await page.getByLabel(copy.theme.legend).selectOption('dark');
  await expect
    .poll(async () =>
      page.evaluate(() => getComputedStyle(document.body).backgroundColor),
    )
    .toBe('rgb(16, 24, 38)');
  await expect(
    page.getByRole('heading', { name: copy.destinations.collect }),
  ).toBeVisible();
});

test.describe('browser high-DPI emulation', () => {
  test.use({
    deviceScaleFactor: 2,
    viewport: { width: 800, height: 600 },
  });

  test('keeps Collect usable at the minimum window', async ({ page }) => {
    await page.goto('/');
    await expect(
      page.getByRole('heading', { name: copy.destinations.collect }),
    ).toBeVisible();
    await expect(
      page.getByRole('navigation', { name: copy.primaryNav }),
    ).toBeVisible();
    const overflow = await page.evaluate(
      () =>
        document.documentElement.scrollWidth >
        document.documentElement.clientWidth + 1,
    );
    expect(overflow).toBe(false);
  });
});

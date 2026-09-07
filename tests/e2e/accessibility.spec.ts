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
    page.getByRole('heading', { name: 'Common problems' }),
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

test('Help topic links, disclosures and responsive layouts are accessible', async ({
  page,
}) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Help', exact: true }).click();
  for (const width of [1280, 800]) {
    await page.setViewportSize({ width, height: width === 1280 ? 800 : 600 });
    for (const theme of ['light', 'dark']) {
      await page.getByLabel(copy.theme.legend).selectOption(theme);
      const link = page.getByRole('link', {
        name: 'Common problems',
        exact: true,
      });
      await link.focus();
      await page.keyboard.press('Enter');
      await expect(
        page.getByRole('heading', { name: 'Common problems', exact: true }),
      ).toBeFocused();
      const summary = page
        .locator('summary')
        .filter({ hasText: 'A report bundle is incomplete' });
      await summary.focus();
      await page.keyboard.press('Enter');
      await expect(summary.locator('..')).toHaveAttribute('open', '');
      await expect(
        summary.locator('..').getByText(/Restore the missing original file/),
      ).toBeVisible();
      await page.keyboard.press('Space');
      await expect(summary.locator('..')).not.toHaveAttribute('open', '');
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth > innerWidth + 1,
        ),
      ).toBe(false);
    }
  }
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '150%';
  });
  await page
    .getByRole('link', {
      name: 'File formats and version details',
      exact: true,
    })
    .click();
  await page
    .getByText('Show file formats, version, and diagnostic codes', {
      exact: true,
    })
    .click();
  await expect(
    page.getByText('unexpected_failure', { exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth > innerWidth + 1,
    ),
  ).toBe(false);
});

test('Collect Device setup focuses Help and disclosures stay usable', async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/');
  await expect(
    page.getByRole('button', { name: copy.deviceSetup }),
  ).toBeVisible();
  await page.getByRole('button', { name: copy.deviceSetup }).click();
  await expect(
    page.getByRole('heading', { name: copy.deviceSetup, exact: true }),
  ).toBeFocused();

  for (const theme of ['light', 'dark']) {
    await page.getByLabel(copy.theme.legend).selectOption(theme);
    const summary = page
      .locator('summary')
      .filter({ hasText: 'Windows / BitBabbler' });
    await summary.focus();
    await page.keyboard.press('Enter');
    await expect(summary.locator('..')).toHaveAttribute('open', '');
    await expect(
      summary.locator('..').getByText('0403:7840').first(),
    ).toBeVisible();
    await page.keyboard.press('Space');
    await expect(summary.locator('..')).not.toHaveAttribute('open', '');
  }

  await page.setViewportSize({ width: 800, height: 600 });
  await expect(
    page.getByRole('heading', { name: copy.deviceSetup, exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth > innerWidth + 1,
    ),
  ).toBe(false);

  await page.evaluate(() => {
    document.documentElement.style.fontSize = '150%';
  });
  const linux = page
    .locator('summary')
    .filter({ hasText: 'Ubuntu-Debian / TrueRNG3' });
  await linux.click();
  await expect(
    linux.locator('..').getByText('04d8:f5fe').first(),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth > innerWidth + 1,
    ),
  ).toBe(false);

  await page.getByRole('button', { name: copy.destinations.collect }).click();
  await expect(
    page.getByRole('heading', { name: copy.destinations.collect }),
  ).toBeVisible();
  await expect(
    page.getByRole('button', { name: copy.refreshSources }),
  ).toBeVisible();
  await expect(
    page.getByRole('button', { name: copy.deviceSetup }),
  ).toBeVisible();
  await expect(page.getByText(copy.chart.empty)).toBeVisible();
});

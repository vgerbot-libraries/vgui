import { expect, test } from '@playwright/test';

const routes = ['', 'docs/', 'components/', 'zh-cn/', 'zh-cn/docs/', 'zh-cn/components/'];

test('all localized routes and base-aware navigation links are generated', async ({ page }) => {
  for (const route of routes) {
    const response = await page.goto(route);
    expect(response?.status(), route).toBe(200);
    await expect(page.locator('h1').first()).toBeVisible();
  }

  await page.goto('');
  const nav = page.locator('.desktop-header-nav');
  await expect(nav.getByRole('link', { name: 'Docs', exact: true })).toHaveAttribute(
    'href',
    '/vgui/docs/',
  );
  await expect(nav.getByRole('link', { name: 'Components', exact: true })).toHaveAttribute(
    'href',
    '/vgui/components/',
  );
  await expect(page.getByRole('link', { name: 'Read the docs' })).toHaveAttribute(
    'href',
    './docs/',
  );
  await expect(page.getByRole('link', { name: 'Browse components' })).toHaveAttribute(
    'href',
    './components/',
  );
  await expect(page.locator('.desktop-preferences').getByRole('link', { name: 'GitHub' })).toHaveAttribute(
    'href',
    'https://github.com/vgerbot-libraries/vgui',
  );

  await page.goto('zh-cn/docs/');
  await expect(nav.getByRole('link', { name: '文档', exact: true })).toHaveAttribute(
    'href',
    '/vgui/zh-cn/docs/',
  );
  await expect(nav.getByRole('link', { name: '组件', exact: true })).toHaveAttribute(
    'href',
    '/vgui/zh-cn/components/',
  );
  await expect(nav.getByRole('link', { name: '文档', exact: true })).toHaveAttribute(
    'aria-current',
    'location',
  );
});

test('header remains within the viewport at intermediate and wide sizes', async ({ page }) => {
  for (const width of [820, 1024, 1280]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto('');
    await expect(page.locator('.desktop-header-nav')).toBeVisible();
    await expect(page.locator('.desktop-preferences')).toBeVisible();
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      `viewport width: ${width}`,
    ).toBe(true);
  }
});

test('desktop resources and version disclosures support keyboard and pointer dismissal', async ({
  page,
}) => {
  await page.goto('');
  const nav = page.locator('.desktop-header-nav');
  const resources = nav.locator('summary').filter({ hasText: 'Resources' });
  const version = nav.locator('summary').filter({ hasText: 'v0.1.0' });

  await resources.focus();
  await page.keyboard.press('Enter');
  await expect(resources).toHaveAttribute('aria-expanded', 'true');
  await page.keyboard.press('Tab');
  const apiDoc = nav.getByRole('link', { name: 'API Doc', exact: true });
  await expect(apiDoc).toBeFocused();
  await expect(apiDoc).toHaveAttribute('href', '/vgui/api/vgui/');
  await expect(nav.getByRole('link', { name: /^Releases/ })).toHaveAttribute(
    'href',
    'https://github.com/vgerbot-libraries/vgui/releases',
  );
  await expect(nav.getByRole('link', { name: /^Issues/ })).toHaveAttribute('target', '_blank');

  await page.keyboard.press('Escape');
  await expect(resources).toHaveAttribute('aria-expanded', 'false');
  await expect(resources).toBeFocused();
  await expect(apiDoc).not.toBeVisible();

  await resources.click();
  await page.locator('h1').first().click();
  await expect(resources).toHaveAttribute('aria-expanded', 'false');

  await resources.click();
  await version.click();
  await expect(resources).toHaveAttribute('aria-expanded', 'false');
  await expect(version).toHaveAttribute('aria-expanded', 'true');
  await expect(nav.locator('.version-item')).toHaveAttribute('aria-current', 'true');
  await expect(nav.locator('.latest-badge')).toHaveText('Latest');
});

test('mobile drawer exposes every navigation destination without horizontal overflow', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('docs/');
  await expect(page.locator('.desktop-header-nav')).toBeHidden();
  await page.locator('button[popovertarget="starlight__sidebar"]').click();
  await expect
    .poll(() => page.locator('#starlight__sidebar').evaluate((element) => element.matches(':popover-open')))
    .toBe(true);

  const nav = page.locator('.mobile-header-nav');
  await expect(nav).toBeVisible();
  await expect(nav.getByRole('link', { name: 'Docs', exact: true })).toBeVisible();
  await expect(nav.getByRole('link', { name: 'Components', exact: true })).toBeVisible();
  await expect(nav.getByRole('link', { name: /^GitHub/ })).toBeVisible();
  await nav.locator('summary').filter({ hasText: 'Resources' }).click();
  await expect(nav.getByRole('link', { name: 'API Doc', exact: true })).toBeVisible();
  await nav.locator('summary').filter({ hasText: 'v0.1.0' }).click();
  await expect(nav.locator('.version-item')).toBeVisible();
  await expect(page.locator('starlight-theme-select select:visible')).toBeVisible();
  await expect(page.locator('starlight-lang-select select:visible')).toBeVisible();
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
  ).toBe(true);
});

test('language switching preserves the current section in both directions', async ({ page }) => {
  for (const route of ['', 'docs/', 'components/']) {
    await page.goto(route);
    await page.locator('.desktop-preferences starlight-lang-select select').selectOption({
      label: '简体中文',
    });
    await expect(page).toHaveURL(new RegExp(`/vgui/zh-cn/${route.replace('/', '\\/')}$`));
    await expect(page.locator('html')).toHaveAttribute('lang', 'zh-CN');

    await page.locator('.desktop-preferences starlight-lang-select select').selectOption({
      label: 'English',
    });
    await expect(page).toHaveURL(new RegExp(`/vgui/${route.replace('/', '\\/')}$`));
    await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  }
});

test('theme choices take effect and explicit preferences survive reloads', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await page.goto('');
  const theme = page.locator('.desktop-preferences starlight-theme-select select');

  await theme.selectOption('dark');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.reload();
  await expect(theme).toHaveValue('dark');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

  await theme.selectOption('auto');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await page.emulateMedia({ colorScheme: 'dark' });
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.reload();
  await expect(theme).toHaveValue('auto');
});

test('Pagefind searches English and Chinese pages but not Rustdoc', async ({ page }) => {
  await page.goto('');
  await page.locator('site-search button[data-open-modal]').click();
  const input = page.locator('.pagefind-ui__search-input');
  await expect(input).toBeVisible();
  await input.fill('experimental');
  await expect(page.locator('.pagefind-ui__result-link').first()).toBeVisible();
  await expect(page.locator('.pagefind-ui__result-link').first()).toHaveAttribute(
    'href',
    /\/vgui\//,
  );

  await input.fill('WindowContext');
  await expect(page.locator('.pagefind-ui__result')).toHaveCount(0);

  await page.keyboard.press('Escape');
  await page.goto('zh-cn/');
  await page.locator('site-search button[data-open-modal]').click();
  const chineseInput = page.locator('.pagefind-ui__search-input');
  await expect(chineseInput).toBeVisible();
  await chineseInput.fill('学习路径');
  await expect(page.locator('.pagefind-ui__result-link').first()).toBeVisible();
  await expect(page.locator('.pagefind-ui__result-link').first()).toHaveAttribute(
    'href',
    /\/vgui\/zh-cn\/docs\//,
  );
});

test('Rustdoc is deployed with working assets and source links', async ({ page, request }) => {
  const response = await page.goto('api/vgui/');
  expect(response?.status()).toBe(200);
  await expect(page.locator('body.rustdoc')).toBeVisible();
  await expect(page.locator('h1')).toContainText('Crate vgui');
  expect(await page.locator('link[rel="stylesheet"]').count()).toBeGreaterThan(0);

  const sourceHref = await page.locator('a[href*="/src/"]').first().getAttribute('href');
  expect(sourceHref).toBeTruthy();
  const sourceResponse = await request.get(new URL(sourceHref!, page.url()).pathname);
  expect(sourceResponse.status()).toBe(200);
});

test('production pages emit no console errors or missing local resources', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('response', (response) => {
    if (response.url().startsWith('http://127.0.0.1:4321/') && response.status() >= 400) {
      errors.push(`${response.status()} ${response.url()}`);
    }
  });

  for (const route of [...routes, 'api/vgui/']) {
    const response = await page.goto(route);
    expect(response?.status(), route).toBe(200);
    await page.waitForLoadState('networkidle');
  }
  expect(errors).toEqual([]);
});

test('navigation respects reduced motion and exposes a keyboard focus indicator', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('');
  const resources = page.locator('.desktop-header-nav summary').filter({ hasText: 'Resources' });
  await expect(resources).toHaveCSS('transition-duration', '0s');

  await resources.focus();
  await page.keyboard.press('Shift+Tab');
  await page.keyboard.press('Tab');
  await expect(resources).toBeFocused();
  await expect(resources).toHaveCSS('outline-style', 'solid');
  await expect(resources).toHaveCSS('outline-width', '2px');
});

import { expect, test } from '@playwright/test';

const docsRoutes = [
  'docs/',
  'docs/installation/',
  'docs/quick-start/',
  'docs/first-reactive-view/',
  'docs/native-and-web/',
  'docs/compatibility-troubleshooting/',
];
const localizedDocsRoutes = [
  ...docsRoutes,
  ...docsRoutes.map((route) => `zh-cn/${route}`),
];
const routes = ['', 'components/', 'zh-cn/', 'zh-cn/components/', ...localizedDocsRoutes];

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

test('Start Here sidebar exposes only completed bilingual documentation', async ({ page }) => {
  const englishLinks = [
    ['Introduction', '/vgui/docs/'],
    ['Installation', '/vgui/docs/installation/'],
    ['Quick Start', '/vgui/docs/quick-start/'],
    ['Your First Reactive View', '/vgui/docs/first-reactive-view/'],
    ['Native & Web', '/vgui/docs/native-and-web/'],
    ['Compatibility & Troubleshooting', '/vgui/docs/compatibility-troubleshooting/'],
  ];
  const chineseLinks = [
    ['简介', '/vgui/zh-cn/docs/'],
    ['安装', '/vgui/zh-cn/docs/installation/'],
    ['快速开始', '/vgui/zh-cn/docs/quick-start/'],
    ['第一个响应式视图', '/vgui/zh-cn/docs/first-reactive-view/'],
    ['原生与 Web', '/vgui/zh-cn/docs/native-and-web/'],
    ['兼容性与故障排查', '/vgui/zh-cn/docs/compatibility-troubleshooting/'],
  ];

  for (const [route, group, links, docsLabel] of [
    ['docs/quick-start/', 'Start Here', englishLinks, 'Docs'],
    ['zh-cn/docs/quick-start/', '从这里开始', chineseLinks, '文档'],
  ] as const) {
    await page.goto(route);
    const sidebar = page.locator('#starlight__sidebar');
    await expect(sidebar.getByText(group, { exact: true })).toBeVisible();
    for (const [label, href] of links) {
      await expect(sidebar.getByRole('link', { name: label, exact: true })).toHaveAttribute(
        'href',
        href,
      );
    }
    await expect(sidebar.getByText('Guides', { exact: true })).toHaveCount(0);
    await expect(sidebar.getByText('Reference', { exact: true })).toHaveCount(0);
    await expect(
      page.locator('.desktop-header-nav').getByRole('link', { name: docsLabel, exact: true }),
    ).toHaveAttribute('aria-current', 'location');
  }

  for (const route of localizedDocsRoutes) {
    await page.goto(route);
    await expect(page.locator('.empty-state')).toHaveCount(0);
  }
});

test('Quick Start includes a complete runnable counter', async ({ page }) => {
  await page.goto('docs/quick-start/');
  const content = page.locator('main');
  await expect(content.locator('pre[data-language="toml"]')).toHaveCount(1);
  await expect(content.locator('pre[data-language="rust"]')).toHaveCount(1);
  await expect(content).toContainText('fn main()');
  await expect(content).toContainText('open_window');
  await expect(content).toContainText('vgui::mount');
  await expect(content).toContainText('cargo +nightly run');
});

test('documentation body links resolve', async ({ page, request }) => {
  const urls = new Set<string>();
  for (const route of localizedDocsRoutes) {
    await page.goto(route);
    const hrefs = await page
      .locator('main a[href]')
      .evaluateAll((links) => links.map((link) => (link as HTMLAnchorElement).href));
    for (const href of hrefs) {
      const url = new URL(href);
      if (url.origin === new URL(page.url()).origin && url.pathname.startsWith('/vgui/')) {
        url.hash = '';
        urls.add(url.toString());
      }
    }
  }

  for (const url of urls) {
    const response = await request.get(url);
    expect(response.status(), url).toBeLessThan(400);
  }
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
  for (const route of ['', 'components/', ...docsRoutes]) {
    await page.goto(route);
    await page.locator('.desktop-preferences starlight-lang-select select').selectOption({
      label: '简体中文',
    });
    await page.waitForURL((url) => url.pathname === `/vgui/zh-cn/${route}`);
    expect(new URL(page.url()).pathname).toBe(`/vgui/zh-cn/${route}`);
    await expect(page.locator('html')).toHaveAttribute('lang', 'zh-CN');

    await page.locator('.desktop-preferences starlight-lang-select select').selectOption({
      label: 'English',
    });
    await page.waitForURL((url) => url.pathname === `/vgui/${route}`);
    expect(new URL(page.url()).pathname).toBe(`/vgui/${route}`);
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

  await input.fill('SharedString');
  await expect(page.locator('.pagefind-ui__result')).toHaveCount(0);

  await page.keyboard.press('Escape');
  await page.goto('zh-cn/');
  await page.locator('site-search button[data-open-modal]').click();
  const chineseInput = page.locator('.pagefind-ui__search-input');
  await expect(chineseInput).toBeVisible();
  await chineseInput.fill('故障排查');
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

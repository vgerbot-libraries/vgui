import { expect, test } from '@playwright/test';

const docsRoutes = [
  'docs/',
  'docs/installation/',
  'docs/quick-start/',
  'docs/first-reactive-view/',
  'docs/native-and-web/',
  'docs/compatibility-troubleshooting/',
  'docs/architecture/',
  'docs/view-macro/',
  'docs/reactivity/',
  'docs/control-flow/',
  'docs/custom-components/',
  'docs/overview/',
  'docs/tailwind-and-variants/',
  'docs/theming-and-animation/',
  'docs/built-in-elements-and-events/',
  'docs/inputs-and-forms/',
  'docs/refs-and-context/',
  'docs/examples/counter/',
  'docs/examples/store-and-lists/',
  'docs/examples/events-and-inputs/',
];
const componentRoutes = [
  'components/',
  'components/actions-and-form-controls/',
  'components/data-display/',
  'components/navigation-and-layout/',
  'components/overlays-and-feedback/',
];
const contentRoutes = [...docsRoutes, ...componentRoutes];
const localizedContentRoutes = [
  ...contentRoutes,
  ...contentRoutes.map((route) => `zh-cn/${route}`),
];
const routes = ['', 'zh-cn/', ...localizedContentRoutes];

test('all localized routes and base-aware navigation links are generated', async ({ page }) => {
  test.setTimeout(120_000);
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

test('sidebar exposes all completed bilingual documentation', async ({ page }) => {
  test.setTimeout(120_000);
  const groups = [
    {
      labels: ['Start Here', '从这里开始'],
      links: [
        ['Introduction', '简介', 'docs/'],
        ['Installation', '安装', 'docs/installation/'],
        ['Quick Start', '快速开始', 'docs/quick-start/'],
        ['Your First Reactive View', '第一个响应式视图', 'docs/first-reactive-view/'],
        ['Native & Web', '原生与 Web', 'docs/native-and-web/'],
        [
          'Compatibility & Troubleshooting',
          '兼容性与故障排查',
          'docs/compatibility-troubleshooting/',
        ],
      ],
    },
    {
      labels: ['Core Concepts', '核心概念'],
      links: [
        ['Architecture', '架构', 'docs/architecture/'],
        ['The view! Macro', 'view! 宏', 'docs/view-macro/'],
        ['Reactivity', '响应式状态', 'docs/reactivity/'],
        ['Control Flow', '控制流', 'docs/control-flow/'],
        ['Custom Components', '自定义组件', 'docs/custom-components/'],
      ],
    },
    {
      labels: ['Styling', '样式'],
      links: [
        ['Overview', '概述', 'docs/overview/'],
        ['Tailwind & Variants', 'Tailwind 与 Variants', 'docs/tailwind-and-variants/'],
        ['Theming & Animation', '主题与动画', 'docs/theming-and-animation/'],
      ],
    },
    {
      labels: ['Elements & State', '元素与状态'],
      links: [
        ['Built-in Elements & Events', '内置元素与事件', 'docs/built-in-elements-and-events/'],
        ['Inputs & Forms', '输入与表单', 'docs/inputs-and-forms/'],
        ['Refs & Context', 'Refs 与 Context', 'docs/refs-and-context/'],
      ],
    },
    {
      labels: ['Examples', '示例'],
      links: [
        ['Counter', '计数器', 'docs/examples/counter/'],
        ['Store & Lists', 'Store 与列表', 'docs/examples/store-and-lists/'],
        ['Events & Inputs', '事件与输入', 'docs/examples/events-and-inputs/'],
      ],
    },
    {
      labels: ['Components', '组件'],
      links: [
        ['Overview', '组件概览', 'components/'],
        ['Actions & Form Controls', '操作与表单控件', 'components/actions-and-form-controls/'],
        ['Data Display', '数据展示', 'components/data-display/'],
        ['Navigation & Layout', '导航与布局', 'components/navigation-and-layout/'],
        ['Overlays & Feedback', '浮层与反馈', 'components/overlays-and-feedback/'],
      ],
    },
  ] as const;

  for (const [route, chinese, docsLabel] of [
    ['docs/quick-start/', false, 'Docs'],
    ['zh-cn/docs/quick-start/', true, '文档'],
  ] as const) {
    await page.goto(route);
    const sidebar = page.locator('#starlight__sidebar');
    for (const group of groups) {
      const groupLabel = sidebar
        .locator('summary')
        .getByText(group.labels[chinese ? 1 : 0], { exact: true });
      await expect(groupLabel).toBeVisible();
      const groupContainer = groupLabel.locator('xpath=ancestor::details[1]');
      for (const link of group.links) {
        await expect(
          groupContainer.getByRole('link', { name: link[chinese ? 1 : 0], exact: true }),
        ).toHaveAttribute('href', `/vgui/${chinese ? 'zh-cn/' : ''}${link[2]}`);
      }
    }
    await expect(
      page.locator('.desktop-header-nav').getByRole('link', { name: docsLabel, exact: true }),
    ).toHaveAttribute('aria-current', 'location');
  }

  for (const route of localizedContentRoutes) {
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
  test.setTimeout(120_000);
  const urls = new Set<string>();
  for (const route of localizedContentRoutes) {
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
  test.setTimeout(120_000);
  for (const route of ['', ...contentRoutes]) {
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
  await expect(page.locator('.pagefind-ui__result-link').first()).toBeVisible();
  await expect(page.locator('.pagefind-ui__result-link[href*="/api/"]')).toHaveCount(0);

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

test('quick start and counter example render interactive WASM', async ({ page }) => {
  test.setTimeout(120_000);
  for (const route of ['docs/quick-start/', 'docs/examples/counter/']) {
    await page.goto(route);

    const example = page.locator('.live-rust-example');
    await expect(example.locator('.expressive-code')).toContainText('single_threaded_web');
    await expect(example.locator('iframe')).toHaveAttribute(
      'src',
      '/vgui/wasm/counter/index.html',
    );
    const frame = page.frameLocator('.live-rust-frame iframe');
    const canvas = frame.locator('canvas');
    await expect(frame.locator('html')).toHaveAttribute('data-vgui-ready', 'true', {
      timeout: 90_000,
    });
    await expect(canvas).toBeVisible();

    await canvas.scrollIntoViewIfNeeded();
    const bounds = await canvas.boundingBox();
    expect(bounds).not.toBeNull();
    const before = await canvas.screenshot();
    const x = bounds!.x + bounds!.width / 2;
    const y = bounds!.y + bounds!.height / 2 + 16;
    await page.mouse.move(x, y, { steps: 8 });
    const hovered = await canvas.screenshot();
    await page.mouse.click(x, y, { delay: 100 });
    await expect
      .poll(async () => hovered.equals(await canvas.screenshot()), { timeout: 15_000 })
      .toBe(false);

    await page.setViewportSize({ width: 390, height: 844 });
    await expect(example).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
      true,
    );
  }
});

test('production pages emit no console errors or missing local resources', async ({ page }) => {
  test.setTimeout(120_000);
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

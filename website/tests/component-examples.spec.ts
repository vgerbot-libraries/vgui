import { expect, test, type Locator, type Page } from '@playwright/test';

test.describe.configure({ mode: 'serial' });

const READY_TIMEOUT_MS = 90_000;
const PIXEL_TIMEOUT_MS = 15_000;
// At 1024px the documentation uses its single-column preview layout, leaving
// enough room for examples with fixed-width controls such as Pagination.
const DESKTOP_VIEWPORT = { width: 1024, height: 900 };
const MOBILE_VIEWPORT = { width: 390, height: 844 };

interface ExampleSpec {
  name: string;
  sourceFunction: string;
}

interface ComponentPageSpec {
  route: string;
  examples: readonly ExampleSpec[];
}

interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface Point {
  x: number;
  y: number;
}

interface Size {
  width: number;
  height: number;
}

type PixelSnapshot = Awaited<ReturnType<Page['screenshot']>>;

const componentPages: readonly ComponentPageSpec[] = [
  {
    route: 'components/',
    examples: [{ name: 'themed_view', sourceFunction: 'fn themed_view()' }],
  },
  {
    route: 'components/actions-and-form-controls/',
    examples: [
      { name: 'click_button', sourceFunction: 'fn click_button()' },
      { name: 'profile_form', sourceFunction: 'fn profile_form()' },
      { name: 'language_select', sourceFunction: 'fn language_select()' },
    ],
  },
  {
    route: 'components/data-display/',
    examples: [
      { name: 'project_card', sourceFunction: 'fn project_card()' },
      { name: 'package_details', sourceFunction: 'fn package_details()' },
      { name: 'team_table', sourceFunction: 'fn team_table()' },
    ],
  },
  {
    route: 'components/navigation-and-layout/',
    examples: [
      { name: 'breadcrumb', sourceFunction: 'fn breadcrumb()' },
      { name: 'pagination', sourceFunction: 'fn pagination()' },
      { name: 'tabs', sourceFunction: 'fn tabs()' },
      { name: 'h_resizable', sourceFunction: 'fn horizontal_split()' },
    ],
  },
  {
    route: 'components/overlays-and-feedback/',
    examples: [
      { name: 'dialog', sourceFunction: 'fn dialog()' },
      { name: 'popover', sourceFunction: 'fn popover()' },
      { name: 'alert', sourceFunction: 'fn alert()' },
      { name: 'progress', sourceFunction: 'fn progress()' },
    ],
  },
];

async function canvasBounds(canvas: Locator): Promise<Box> {
  const bounds = await canvas.boundingBox();
  expect(bounds, 'canvas must have a layout box').not.toBeNull();
  expect(bounds!.width, 'canvas CSS width must be non-zero').toBeGreaterThan(0);
  expect(bounds!.height, 'canvas CSS height must be non-zero').toBeGreaterThan(0);
  return bounds!;
}

async function canvasSize(canvas: Locator): Promise<Size> {
  const { width, height } = await canvasBounds(canvas);
  return { width, height };
}

async function afterTwoAnimationFrames(canvas: Locator): Promise<void> {
  await canvas.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      }),
  );
}

async function canvasPixels(canvas: Locator, region?: Box): Promise<PixelSnapshot> {
  await canvas.scrollIntoViewIfNeeded();
  await afterTwoAnimationFrames(canvas);
  const bounds = await canvasBounds(canvas);
  if (!region) return canvas.screenshot();

  expect(region.x).toBeGreaterThanOrEqual(0);
  expect(region.y).toBeGreaterThanOrEqual(0);
  expect(region.x + region.width).toBeLessThanOrEqual(bounds.width);
  expect(region.y + region.height).toBeLessThanOrEqual(bounds.height);
  return canvas.page().screenshot({
    clip: {
      x: bounds.x + region.x,
      y: bounds.y + region.y,
      width: region.width,
      height: region.height,
    },
  });
}

async function expectCanvasChange(
  canvas: Locator,
  before: PixelSnapshot,
  region?: Box,
): Promise<PixelSnapshot> {
  let latest = before;
  await expect
    .poll(
      async () => {
        latest = await canvasPixels(canvas, region);
        return latest.equals(before);
      },
      {
        timeout: PIXEL_TIMEOUT_MS,
        intervals: [100, 250, 500, 1_000],
        message: 'the targeted canvas region should repaint after the interaction',
      },
    )
    .toBe(false);
  return latest;
}

async function moveToCanvasPoint(
  page: Page,
  canvas: Locator,
  point: Point,
): Promise<Box> {
  await canvas.scrollIntoViewIfNeeded();
  const bounds = await canvasBounds(canvas);
  expect(point.x).toBeGreaterThanOrEqual(0);
  expect(point.x).toBeLessThanOrEqual(bounds.width);
  expect(point.y).toBeGreaterThanOrEqual(0);
  expect(point.y).toBeLessThanOrEqual(bounds.height);
  await page.mouse.move(bounds.x + point.x, bounds.y + point.y, { steps: 4 });
  return bounds;
}

async function clickAndExpectChange(
  page: Page,
  canvas: Locator,
  point: Point,
  region?: Box,
): Promise<PixelSnapshot> {
  const bounds = await moveToCanvasPoint(page, canvas, point);
  // Capture after hover so a hover-only repaint cannot satisfy the click assertion.
  const before = await canvasPixels(canvas, region);
  await page.mouse.click(bounds.x + point.x, bounds.y + point.y, { delay: 80 });
  return expectCanvasChange(canvas, before, region);
}

function centeredRegion(size: Size, width: number, height: number): Box {
  const clippedWidth = Math.min(width, size.width);
  const clippedHeight = Math.min(height, size.height);
  return {
    x: (size.width - clippedWidth) / 2,
    y: (size.height - clippedHeight) / 2,
    width: clippedWidth,
    height: clippedHeight,
  };
}

async function exerciseProfileForm(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  const inputRegion = { x: 16, y: 16, width: size.width - 32, height: 36 };
  const inputPoint = { x: 48, y: 34 };

  // profile_form has a 16px padding and a 36px-high TextField in its first row.
  const beforeInput = await canvasPixels(canvas, inputRegion);
  const inputBounds = await moveToCanvasPoint(page, canvas, inputPoint);
  await page.mouse.click(inputBounds.x + inputPoint.x, inputBounds.y + inputPoint.y);
  await page.keyboard.press('Control+A');
  await page.keyboard.type('Ada Lovelace');
  // Leave the input so its blinking caret cannot be mistaken for entered text.
  await page.keyboard.press('Tab');
  await expectCanvasChange(canvas, beforeInput, inputRegion);

  // The second row starts after 36px + the 12px flex gap; Switch is 40x22px.
  await clickAndExpectChange(
    page,
    canvas,
    { x: 36, y: 75 },
    { x: 16, y: 64, width: Math.min(280, size.width - 32), height: 22 },
  );
}

async function exerciseLanguageSelect(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  const popupRegion = { x: 16, y: 16, width: size.width - 32, height: 190 };
  const triggerTextRegion = {
    x: 28,
    y: 16,
    width: Math.min(180, size.width - 56),
    height: 32,
  };
  const triggerPoint = { x: size.width / 2, y: 32 };

  // Select starts at the 16px top padding and fills the available width.
  const bounds = await moveToCanvasPoint(page, canvas, triggerPoint);
  const initial = await canvasPixels(canvas, popupRegion);
  const initialTriggerText = await canvasPixels(canvas, triggerTextRegion);
  await page.mouse.click(bounds.x + triggerPoint.x, bounds.y + triggerPoint.y, { delay: 80 });
  const opened = await expectCanvasChange(canvas, initial, popupRegion);

  // Rust is initially selected. ArrowDown highlights TypeScript and Enter commits it.
  await page.keyboard.press('ArrowDown');
  const highlighted = await expectCanvasChange(canvas, opened, popupRegion);
  await page.keyboard.press('Enter');
  await expectCanvasChange(canvas, highlighted, popupRegion);
  await expect
    .poll(async () => (await canvasPixels(canvas, triggerTextRegion)).equals(initialTriggerText), {
      timeout: PIXEL_TIMEOUT_MS,
      message: 'the select trigger should change from Rust to TypeScript',
    })
    .toBe(false);
}

async function exerciseTeamTable(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  // team_table is centered as a 314px-wide group: 16px padding on each side of
  // a 282px table (120px + 160px columns + 2px border). Its label is one 24px
  // line, followed by an 8px gap and the 124px-high table.
  const tableLeft = (size.width - 314) / 2 + 16;
  const tableTop = (size.height - 188) / 2 + 48;
  const firstRowRegion = { x: tableLeft, y: tableTop + 33, width: 280, height: 30 };
  const nameHeaderRegion = { x: tableLeft, y: tableTop, width: 120, height: 33 };

  await clickAndExpectChange(
    page,
    canvas,
    { x: tableLeft + 60, y: tableTop + 48 },
    firstRowRegion,
  );
  await clickAndExpectChange(
    page,
    canvas,
    { x: tableLeft + 60, y: tableTop + 16 },
    nameHeaderRegion,
  );
}

async function exercisePagination(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  // Page 5 renders nine 32px items (previous, seven page-window entries, next)
  // with eight 4px gaps, for a centered 320x32px navigation row.
  const navLeft = (size.width - 320) / 2;
  const pageEntriesRegion = { x: navLeft + 36, y: (size.height - 32) / 2, width: 252, height: 32 };
  await clickAndExpectChange(
    page,
    canvas,
    { x: navLeft + 304, y: size.height / 2 },
    pageEntriesRegion,
  );
}

async function exerciseTabs(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  // The centered TabsList has 4px padding and a 4px gap. Account is roughly
  // 76px wide and Password roughly 82px; this lands in Password, not the panel.
  // The 40px list is 16px above the canvas center for the 72px-high Tabs group.
  await clickAndExpectChange(
    page,
    canvas,
    { x: size.width / 2 + 40, y: size.height / 2 - 16 },
    centeredRegion(size, 280, 110),
  );
}

async function exerciseHorizontalResizable(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  // h_resizable is a centered 320x320px split. Its initial 120px left pane is
  // followed by a 6px divider, so the divider center is splitLeft + 123px.
  const splitRegion = centeredRegion(size, 320, 320);
  const divider = { x: splitRegion.x + 123, y: size.height / 2 };
  const bounds = await moveToCanvasPoint(page, canvas, divider);
  const before = await canvasPixels(canvas, splitRegion);
  const startX = bounds.x + divider.x;
  const startY = bounds.y + divider.y;
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  try {
    await page.mouse.move(startX + 64, startY, { steps: 8 });
    await expectCanvasChange(canvas, before, splitRegion);
  } finally {
    await page.mouse.up();
  }
}

async function exerciseDialog(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  // The only non-overlay element is the centered Delete item trigger.
  const opened = await clickAndExpectChange(page, canvas, {
    x: size.width / 2,
    y: size.height / 2,
  });
  await page.keyboard.press('Escape');
  await expectCanvasChange(canvas, opened);
}

async function exercisePopover(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  // Project settings is the sole centered anchor; its floating content opens below it.
  const trigger = { x: size.width / 2, y: size.height / 2 };
  const opened = await clickAndExpectChange(page, canvas, trigger);
  const bounds = await moveToCanvasPoint(page, canvas, trigger);
  await page.mouse.click(bounds.x + trigger.x, bounds.y + trigger.y, { delay: 80 });
  await expectCanvasChange(canvas, opened);
}

async function exerciseProgress(page: Page, canvas: Locator): Promise<void> {
  const size = await canvasSize(canvas);
  const contentWidth = Math.min(448, size.width - 32);
  // The centered column is progress(8px) + gap(16px) + Advance(button height).
  // Therefore the button center is always 12px below the column/canvas center.
  // Crop above it so only the progress track, not button hover, proves the update.
  const trackRegion = {
    x: (size.width - contentWidth) / 2,
    y: size.height / 2 - 40,
    width: contentWidth,
    height: 32,
  };
  await clickAndExpectChange(
    page,
    canvas,
    { x: size.width / 2, y: size.height / 2 + 12 },
    trackRegion,
  );
}

async function exerciseExample(page: Page, name: string, canvas: Locator): Promise<void> {
  switch (name) {
    case 'click_button': {
      const size = await canvasSize(canvas);
      // The sole 36px-high Button is centered by the example's flex container.
      await clickAndExpectChange(
        page,
        canvas,
        { x: size.width / 2, y: size.height / 2 },
        centeredRegion(size, 260, 72),
      );
      break;
    }
    case 'profile_form':
      await exerciseProfileForm(page, canvas);
      break;
    case 'language_select':
      await exerciseLanguageSelect(page, canvas);
      break;
    case 'team_table':
      await exerciseTeamTable(page, canvas);
      break;
    case 'pagination':
      await exercisePagination(page, canvas);
      break;
    case 'tabs':
      await exerciseTabs(page, canvas);
      break;
    case 'h_resizable':
      await exerciseHorizontalResizable(page, canvas);
      break;
    case 'dialog':
      await exerciseDialog(page, canvas);
      break;
    case 'popover':
      await exercisePopover(page, canvas);
      break;
    case 'progress':
      await exerciseProgress(page, canvas);
      break;
  }
}

for (const componentPage of componentPages) {
  test(`${componentPage.route} keeps bilingual source, previews, interactions, and mobile layout in sync`, async ({
    page,
  }) => {
    test.setTimeout(900_000);

    for (const localePrefix of ['', 'zh-cn/']) {
      const route = `${localePrefix}${componentPage.route}`;
      await page.setViewportSize(DESKTOP_VIEWPORT);
      const response = await page.goto(route);
      expect(response?.status(), route).toBe(200);

      const examples = page.locator('[data-example]');
      await expect(examples, route).toHaveCount(componentPage.examples.length);
      expect(
        await examples.evaluateAll((elements) =>
          elements.map((element) => element.getAttribute('data-example')),
        ),
        `${route} data-example order`,
      ).toEqual(componentPage.examples.map(({ name }) => name));

      for (const { name, sourceFunction } of componentPage.examples) {
        const example = page.locator(`[data-example="${name}"]`);
        await expect(example.locator('.expressive-code'), `${route} ${name} source`).toContainText(
          sourceFunction,
        );
        await expect(example.locator('iframe'), `${route} ${name} iframe`).toHaveAttribute(
          'src',
          `/vgui/wasm/index.html?example=${name}`,
        );

        // Lazy iframes initialize only after their preview approaches the viewport.
        await example.scrollIntoViewIfNeeded();
        const frame = example.frameLocator('iframe');
        const canvas = frame.locator('canvas');
        await expect(frame.locator('html'), `${route} ${name} readiness`).toHaveAttribute(
          'data-vgui-ready',
          'true',
          { timeout: READY_TIMEOUT_MS },
        );
        await expect(canvas, `${route} ${name} canvas`).toBeVisible();
        const dimensions = await canvas.evaluate((element) => {
          const canvasElement = element as HTMLCanvasElement;
          const bounds = canvasElement.getBoundingClientRect();
          return {
            cssWidth: bounds.width,
            cssHeight: bounds.height,
            bufferWidth: canvasElement.width,
            bufferHeight: canvasElement.height,
          };
        });
        expect(dimensions.cssWidth, `${route} ${name} CSS width`).toBeGreaterThan(0);
        expect(dimensions.cssHeight, `${route} ${name} CSS height`).toBeGreaterThan(0);
        expect(dimensions.bufferWidth, `${route} ${name} buffer width`).toBeGreaterThan(0);
        expect(dimensions.bufferHeight, `${route} ${name} buffer height`).toBeGreaterThan(0);

        if (localePrefix === '') {
          await exerciseExample(page, name, canvas);
        }
      }

      await page.setViewportSize(MOBILE_VIEWPORT);
      for (const { name } of componentPage.examples) {
        const example = page.locator(`[data-example="${name}"]`);
        await example.scrollIntoViewIfNeeded();
        await expect(example.locator('.live-rust-preview'), `${route} ${name} mobile preview`).toBeVisible();
        await expect(example.locator('iframe'), `${route} ${name} mobile iframe`).toBeVisible();
      }
      await expect
        .poll(
          () => page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
          { message: `${route} must not overflow the 390px document viewport` },
        )
        .toBe(true);
    }
  });
}

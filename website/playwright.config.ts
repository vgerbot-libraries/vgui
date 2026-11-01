import { defineConfig } from '@playwright/test';

// Keep local preview traffic direct even when the development machine uses an HTTP proxy.
process.env.NO_PROXY = [process.env.NO_PROXY, '127.0.0.1,localhost'].filter(Boolean).join(',');
process.env.no_proxy = process.env.NO_PROXY;

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  reporter: 'list',
  use: {
    baseURL: 'http://127.0.0.1:4321/vgui/',
    browserName: 'chromium',
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'npm run preview -- --host 127.0.0.1 --port 4321 --ignore-lock',
    url: 'http://127.0.0.1:4321/vgui/',
    reuseExistingServer: !process.env.CI,
  },
});

import { defineConfig, devices } from '@playwright/test';

// End-to-end tests run against the full stack booted by `scripts/e2e-stack.mjs`
// (PostgreSQL + the API serving the built Elm SPA on a single origin — ADR 0011).
const PORT = process.env.E2E_PORT ?? '8081';
const baseURL = `http://localhost:${PORT}`;

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: 'html',
  use: {
    baseURL,
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'node scripts/e2e-stack.mjs',
    url: baseURL,
    reuseExistingServer: false,
    timeout: 180_000,
  },
  // `npm test` runs chromium (the gate); `npm run test:all` runs every browser.
  // Firefox/WebKit need their system libraries (`npx playwright install-deps`).
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
});

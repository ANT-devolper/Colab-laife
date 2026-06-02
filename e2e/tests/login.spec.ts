import { test, expect, request } from '@playwright/test';

// First end-to-end happy path: a freshly provisioned admin logs in through the
// Elm SPA and lands on the (empty) read-only directory. The stack is booted by
// the `webServer` in playwright.config.ts, so this test provisions its own tenant
// through the API (there is no sign-up UI yet). Each project provisions a distinct
// tenant (its name doubles as the schema slug) so the parallel runs stay isolated.

const ADMIN_PASSWORD = 's3cret-pass';

test('admin logs in and sees the empty directory', async ({ page, baseURL }, testInfo) => {
  const slug = `acme_${testInfo.project.name}`.toLowerCase();
  // The email domain must be a valid hostname (no underscores), so keep the
  // per-project uniqueness in the local part; emails are globally unique.
  const adminEmail = `admin-${testInfo.project.name.toLowerCase()}@acme.test`;

  // Provision the tenant + admin through the API.
  const api = await request.newContext({ baseURL });
  const provision = await api.post('/organizations', {
    data: {
      name: slug,
      admin: { name: 'Admin', email: adminEmail, password: ADMIN_PASSWORD },
    },
  });
  expect(provision.status()).toBe(201);
  await api.dispose();

  // Sign in through the SPA.
  await page.goto('/');
  await page.getByLabel('Email').fill(adminEmail);
  await page.getByLabel('Password').fill(ADMIN_PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();

  // The authenticated directory renders, with every list empty.
  await expect(page.getByRole('heading', { name: 'Directory' })).toBeVisible();
  await expect(page.getByText('No collaborators yet.')).toBeVisible();
  await expect(page.getByText('No sectors yet.')).toBeVisible();
  await expect(page.getByText('No roles yet.')).toBeVisible();
});

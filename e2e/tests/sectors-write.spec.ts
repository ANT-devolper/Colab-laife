import { test, expect, request } from '@playwright/test';

// End-to-end of the sector write flow in the SPA: a freshly provisioned admin
// signs in, creates a sector through the form, renames it inline, then
// deactivates it. The stack is booted by the `webServer` in playwright.config.ts;
// each project provisions its own tenant (distinct schema slug) for isolation.

const ADMIN_PASSWORD = 's3cret-pass';

test('admin creates, edits and deactivates a sector', async ({ page, baseURL }, testInfo) => {
  const slug = `sectors_${testInfo.project.name}`.toLowerCase();
  const adminEmail = `admin-sectors-${testInfo.project.name.toLowerCase()}@acme.test`;

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
  await expect(page.getByRole('heading', { name: 'Directory' })).toBeVisible();

  // Create a sector.
  await page.getByLabel('New sector name').fill('Engineering');
  await page.getByRole('button', { name: 'Create sector' }).click();
  await expect(page.getByRole('cell', { name: 'Engineering', exact: true })).toBeVisible();

  // Rename it inline.
  await page.getByRole('row', { name: /Engineering/ }).getByRole('button', { name: 'Edit' }).click();
  await page.getByLabel('Edit sector name').fill('Platform');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByRole('cell', { name: 'Platform', exact: true })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'Engineering', exact: true })).toHaveCount(0);

  // Deactivate it — it leaves the (active-only) list.
  await page.getByRole('row', { name: /Platform/ }).getByRole('button', { name: 'Deactivate' }).click();
  await expect(page.getByRole('cell', { name: 'Platform', exact: true })).toHaveCount(0);
});

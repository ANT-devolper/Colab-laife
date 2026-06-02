import { test, expect, request } from '@playwright/test';

// End-to-end of the role write flow in the SPA: a freshly provisioned admin
// signs in, creates a role through the form, renames it (edit), then deactivates
// it. The stack is booted by the `webServer` in playwright.config.ts; each
// project provisions its own tenant (distinct schema slug) for isolation.

const ADMIN_PASSWORD = 's3cret-pass';

test('admin creates, edits and deactivates a role', async ({ page, baseURL }, testInfo) => {
  const slug = `roles_${testInfo.project.name}`.toLowerCase();
  const adminEmail = `admin-roles-${testInfo.project.name.toLowerCase()}@acme.test`;

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

  // Create a role (name plus one optional field).
  await page.getByLabel('Role name').fill('Backend Engineer');
  await page.getByLabel('Objective').fill('Build the API');
  await page.getByRole('button', { name: 'Create role' }).click();
  await expect(page.getByRole('cell', { name: 'Backend Engineer', exact: true })).toBeVisible();

  // Edit it — the form pre-fills, rename and save.
  await page.getByRole('row', { name: /Backend Engineer/ }).getByRole('button', { name: 'Edit' }).click();
  await expect(page.getByLabel('Role name')).toHaveValue('Backend Engineer');
  await page.getByLabel('Role name').fill('Platform Engineer');
  await page.getByRole('button', { name: 'Save role' }).click();
  await expect(page.getByRole('cell', { name: 'Platform Engineer', exact: true })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'Backend Engineer', exact: true })).toHaveCount(0);

  // Deactivate it — it leaves the (active-only) list.
  await page.getByRole('row', { name: /Platform Engineer/ }).getByRole('button', { name: 'Deactivate' }).click();
  await expect(page.getByRole('cell', { name: 'Platform Engineer', exact: true })).toHaveCount(0);
});

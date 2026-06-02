import { test, expect, request } from '@playwright/test';

// End-to-end of the collaborator write flow in the SPA: a freshly provisioned
// admin signs in and creates a collaborator through the form, picking a sector
// and role from the dropdowns, then renames (edits) and deactivates it. The
// sector and role are seeded through the API before sign-in so they are present
// in the dropdowns when the page loads. The stack is booted by the `webServer`
// in playwright.config.ts; each project provisions its own tenant for isolation.

const ADMIN_PASSWORD = 's3cret-pass';

test('admin creates, edits and deactivates a collaborator', async ({ page, baseURL }, testInfo) => {
  const slug = `collaborators_${testInfo.project.name}`.toLowerCase();
  const adminEmail = `admin-collab-${testInfo.project.name.toLowerCase()}@acme.test`;

  // Provision the tenant + admin, then sign in via the API to seed a sector and
  // a role (so they appear in the collaborator form's dropdowns).
  const api = await request.newContext({ baseURL });
  const provision = await api.post('/organizations', {
    data: {
      name: slug,
      admin: { name: 'Admin', email: adminEmail, password: ADMIN_PASSWORD },
    },
  });
  expect(provision.status()).toBe(201);

  const login = await api.post('/auth/login', {
    data: { email: adminEmail, password: ADMIN_PASSWORD },
  });
  expect(login.status()).toBe(200);
  const token = (await login.json()).token as string;
  const auth = { Authorization: `Bearer ${token}` };

  const sector = await api.post('/sectors', { headers: auth, data: { name: 'Engineering' } });
  expect(sector.status()).toBe(201);
  const role = await api.post('/roles', { headers: auth, data: { name: 'Engineer' } });
  expect(role.status()).toBe(201);
  await api.dispose();

  // Sign in through the SPA.
  await page.goto('/');
  await page.getByLabel('Email').fill(adminEmail);
  await page.getByLabel('Password').fill(ADMIN_PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('heading', { name: 'Directory' })).toBeVisible();

  // Create a collaborator, picking the seeded sector and role.
  await page.getByLabel('Collaborator name').fill('Alice');
  await page.getByLabel('Collaborator sector').selectOption({ label: 'Engineering' });
  await page.getByLabel('Collaborator role').selectOption({ label: 'Engineer' });
  await page.getByLabel('Collaborator email').fill('alice@acme.test');
  await page.getByRole('button', { name: 'Create collaborator' }).click();
  await expect(page.getByRole('cell', { name: 'Alice', exact: true })).toBeVisible();

  // Edit it — the form pre-fills, rename and save.
  await page.getByRole('row', { name: /Alice/ }).getByRole('button', { name: 'Edit' }).click();
  await expect(page.getByLabel('Collaborator name')).toHaveValue('Alice');
  await page.getByLabel('Collaborator name').fill('Alice Smith');
  await page.getByRole('button', { name: 'Save collaborator' }).click();
  await expect(page.getByRole('cell', { name: 'Alice Smith', exact: true })).toBeVisible();

  // Deactivate it — it leaves the (active-only) list.
  await page.getByRole('row', { name: /Alice Smith/ }).getByRole('button', { name: 'Deactivate' }).click();
  await expect(page.getByRole('cell', { name: 'Alice Smith', exact: true })).toHaveCount(0);
});

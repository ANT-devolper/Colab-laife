import { test, expect, request } from '@playwright/test';

// End-to-end of the annotation write flow in the SPA: a freshly provisioned admin
// signs in, opens the Annotations tab, picks a collaborator (seeded via the API),
// creates an annotation, checks that the conditional amount-of-days field appears,
// edits the score type, then deactivates it. The stack is booted by the `webServer`
// in playwright.config.ts; each project provisions its own tenant for isolation.

const ADMIN_PASSWORD = 's3cret-pass';

test('admin creates, edits and deactivates an annotation', async ({ page, baseURL }, testInfo) => {
  const slug = `annotations_${testInfo.project.name}`.toLowerCase();
  const adminEmail = `admin-annot-${testInfo.project.name.toLowerCase()}@acme.test`;

  // Provision the tenant + admin, sign in via the API, and seed a collaborator.
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

  const collaborator = await api.post('/collaborators', {
    headers: { Authorization: `Bearer ${token}` },
    data: { name: 'Alice' },
  });
  expect(collaborator.status()).toBe(201);
  await api.dispose();

  // Sign in through the SPA and open the Annotations tab.
  await page.goto('/');
  await page.getByLabel('Email').fill(adminEmail);
  await page.getByLabel('Password').fill(ADMIN_PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('heading', { name: 'Directory' })).toBeVisible();
  await page.getByRole('button', { name: 'Annotations' }).click();

  // Pick the collaborator and create an annotation.
  await page.getByLabel('Annotation collaborator').selectOption({ label: 'Alice' });
  await page.getByLabel('Note date', { exact: true }).fill('2026-06-02');
  await page.getByLabel('Score 1 number').fill('2');
  await page.getByLabel('Score 1 type').fill('positive');

  // The amount-of-days input is conditional on the checkbox.
  await expect(page.getByLabel('Amount of days', { exact: true })).toHaveCount(0);
  await page.getByLabel('Ask amount of days').check();
  await expect(page.getByLabel('Amount of days', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Create annotation' }).click();
  await expect(page.getByRole('cell', { name: 'positive', exact: true })).toBeVisible();

  // Edit the score type.
  await page.getByRole('row', { name: /positive/ }).getByRole('button', { name: 'Edit' }).click();
  await expect(page.getByLabel('Score 1 type')).toHaveValue('positive');
  await page.getByLabel('Score 1 type').fill('attention');
  await page.getByRole('button', { name: 'Save annotation' }).click();
  await expect(page.getByRole('cell', { name: 'attention', exact: true })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'positive', exact: true })).toHaveCount(0);

  // Deactivate it — it leaves the (active-only) list.
  await page.getByRole('row', { name: /attention/ }).getByRole('button', { name: 'Deactivate' }).click();
  await expect(page.getByRole('cell', { name: 'attention', exact: true })).toHaveCount(0);
});

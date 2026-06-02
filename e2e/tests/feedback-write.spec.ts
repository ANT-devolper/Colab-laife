import { test, expect, request } from '@playwright/test';

// End-to-end of the feedback write flow in the SPA: a freshly provisioned admin
// signs in, opens the Feedback tab, picks a collaborator (seeded via the API),
// creates a feedback, edits its status, then deactivates it. The stack is booted
// by the `webServer` in playwright.config.ts; each project provisions its own
// tenant (distinct schema slug) for isolation.

const ADMIN_PASSWORD = 's3cret-pass';

test('admin creates, edits and deactivates a feedback', async ({ page, baseURL }, testInfo) => {
  const slug = `feedback_${testInfo.project.name}`.toLowerCase();
  const adminEmail = `admin-feedback-${testInfo.project.name.toLowerCase()}@acme.test`;

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
  const auth = { Authorization: `Bearer ${token}` };

  const collaborator = await api.post('/collaborators', { headers: auth, data: { name: 'Alice' } });
  expect(collaborator.status()).toBe(201);
  await api.dispose();

  // Sign in through the SPA and open the Feedback tab.
  await page.goto('/');
  await page.getByLabel('Email').fill(adminEmail);
  await page.getByLabel('Password').fill(ADMIN_PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('heading', { name: 'Directory' })).toBeVisible();
  await page.getByRole('button', { name: 'Feedback' }).click();

  // Pick the collaborator and create a feedback.
  await page.getByLabel('Feedback collaborator').selectOption({ label: 'Alice' });
  await page.getByLabel('Feedback date', { exact: true }).fill('2026-06-02');
  await page.getByLabel('Feedback status').fill('open');
  await page.getByRole('button', { name: 'Create feedback' }).click();
  await expect(page.getByRole('cell', { name: 'open', exact: true })).toBeVisible();

  // Edit its status.
  await page.getByRole('row', { name: /open/ }).getByRole('button', { name: 'Edit' }).click();
  await expect(page.getByLabel('Feedback date', { exact: true })).toHaveValue('2026-06-02');
  await page.getByLabel('Feedback status').fill('reviewed');
  await page.getByRole('button', { name: 'Save feedback' }).click();
  await expect(page.getByRole('cell', { name: 'reviewed', exact: true })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'open', exact: true })).toHaveCount(0);

  // Open it and manage the expectation contract: add a goal, mark it done, remove it.
  await page.getByRole('row', { name: /reviewed/ }).getByRole('button', { name: 'Open' }).click();
  await expect(page.getByRole('heading', { name: 'Expectation contract' })).toBeVisible();
  await page.getByLabel('New goal').fill('Ship the SDK');
  await page.getByRole('button', { name: 'Add goal' }).click();
  await expect(page.getByRole('checkbox', { name: 'Ship the SDK' })).toBeVisible();
  await page.getByRole('checkbox', { name: 'Ship the SDK' }).check();
  await expect(page.getByRole('checkbox', { name: 'Ship the SDK' })).toBeChecked();
  await page.getByRole('listitem').filter({ hasText: 'Ship the SDK' }).getByRole('button', { name: 'Remove' }).click();
  await expect(page.getByRole('checkbox', { name: 'Ship the SDK' })).toHaveCount(0);

  // Deactivate the feedback — it leaves the (active-only) list.
  await page.getByRole('row', { name: /reviewed/ }).getByRole('button', { name: 'Deactivate' }).click();
  await expect(page.getByRole('cell', { name: 'reviewed', exact: true })).toHaveCount(0);
});

// Console flows (claim, sign-in with TOTP, invitation, role change, inactivity) and the
// accessibility of every page in the three languages.
const { test, expect } = require('@playwright/test');
const { completeLink, expectAccessible } = require('./helpers');

const OWNER = 'owner@example.com';
const AUDITOR = 'auditor@example.com';
const PASSWORD = 'a long and unusual passphrase 42';
const LANGUAGES = [
  ['en', 'English'],
  ['fr-CA', 'Français (Canada)'],
  ['pt-BR', 'Português (Brasil)'],
];

test.describe.configure({ mode: 'serial' });

let owner;
let ownerContext;
let ownerPage;
let organizationUrl;
let pendingInvitation;

test.beforeAll(async ({ browser }) => {
  ownerContext = await browser.newContext();
  ownerPage = await ownerContext.newPage();
});

test.afterAll(async () => {
  await ownerContext.close();
});

test('an uninitialized Hub explains how to initialize it', async ({ page }) => {
  await page.goto(process.env.HUB_EMPTY_URL);
  await expect(page.getByRole('heading', { name: 'This Hub is not initialized yet' })).toBeVisible();
  await expect(page.getByText('levix-hub init')).toBeVisible();
});

test('the instance owner claims the Hub with the setup link', async () => {
  owner = await completeLink(ownerPage, process.env.HUB_SETUP_LINK, 'Owner', PASSWORD);
  await expect(ownerPage.getByText(OWNER)).toBeVisible();
});

test('a used setup link is refused', async ({ page }) => {
  await page.goto(process.env.HUB_SETUP_LINK);
  await expect(page.getByRole('alert')).toBeVisible();
});

test('the owner creates an organization and invites an auditor', async ({ browser }) => {
  const page = ownerPage;
  await page.getByLabel('Name', { exact: true }).fill('Acme');
  await page.getByLabel('Short name').fill('acme');
  await page.getByRole('button', { name: 'Create' }).click();
  await page.getByRole('link', { name: 'Acme' }).click();
  await expect(page.getByRole('heading', { name: 'Acme' })).toBeVisible();
  organizationUrl = page.url();

  await page.getByLabel('E-mail').fill(AUDITOR);
  await page.locator('#member-role').selectOption('auditor');
  await page.getByRole('button', { name: 'Add', exact: true }).click();
  const link = (await page.getByTestId('issued-link').textContent()).trim();
  expect(link).toContain('/#/link/lvxl_');

  // The auditor accepts the invitation in their own browser.
  const context = await browser.newContext();
  const auditorPage = await context.newPage();
  await completeLink(auditorPage, link, 'Auditor', PASSWORD);
  await auditorPage.getByRole('link', { name: 'Acme' }).click();
  await expect(auditorPage.getByRole('cell', { name: /auditor@example.com/ })).toBeVisible();
  await expect(auditorPage.getByRole('heading', { name: 'Manage the organization' })).toHaveCount(0);
  await expect(auditorPage.getByRole('link', { name: 'Audit' })).toBeVisible();
  await context.close();
});

test('the owner changes the auditor to read-only', async () => {
  const page = ownerPage;
  await page.goto(organizationUrl);
  await page.getByLabel(`Role of ${AUDITOR}`).selectOption('read_only');
  const row = page.getByRole('row', { name: new RegExp(AUDITOR) });
  await row.getByRole('button', { name: 'Change role' }).click();
  await expect(row.getByRole('cell', { name: 'Read-only' })).toBeVisible();

  // A second invitation stays pending, for the scan of the link page.
  await page.getByLabel('E-mail').fill('pending@example.com');
  await page.getByRole('button', { name: 'Add', exact: true }).click();
  await expect(page.getByText('pending@example.com was invited')).toBeVisible();
  pendingInvitation = (await page.getByTestId('issued-link').textContent()).trim();
});

test('the owner signs out and signs in again with a new code', async () => {
  const page = ownerPage;
  await page.getByRole('button', { name: 'Sign out' }).click();
  await expect(page.getByRole('heading', { name: 'Sign in' })).toBeVisible();
  await page.getByLabel('E-mail').fill(OWNER);
  await page.getByLabel('Password').fill('wrong password');
  await page.getByLabel('Authenticator code').fill('000000');
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('alert')).toBeVisible();
  await page.getByLabel('Password').fill(PASSWORD);
  await page.getByLabel('Authenticator code').fill(await owner.next());
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('heading', { name: 'Organizations', level: 1 })).toBeVisible();
});

test('the audit lists the changes and its chain verifies', async () => {
  const page = ownerPage;
  await page.getByRole('link', { name: 'Audit' }).click();
  await expect(page.getByRole('cell', { name: 'organization.create' })).toBeVisible();
  await page.getByRole('button', { name: 'Verify the chain' }).click();
  await expect(page.getByText(/The audit chain is intact/)).toBeVisible();
});

test('an API token is shown once', async () => {
  const page = ownerPage;
  await page.getByRole('link', { name: 'API tokens' }).click();
  await page.getByLabel('Token name').fill('ci');
  await page.getByRole('button', { name: 'Create a token' }).click();
  await expect(page.getByTestId('token-secret')).toContainText('lvxh_');
  await expect(page.getByRole('cell', { name: 'ci', exact: true })).toBeVisible();
});

test('every page is accessible in every language', async ({ browser }) => {
  const page = ownerPage;
  const pages = [
    ['organizations', '#/organizations'],
    ['organization', new URL(organizationUrl).hash],
    ['audit', '#/audit'],
    ['profile', '#/profile'],
    ['sessions', '#/sessions'],
    ['tokens', '#/tokens'],
    ['instance', '#/instance'],
    ['not found', '#/nowhere'],
  ];
  for (const [language, label] of LANGUAGES) {
    await page.getByLabel(/Language|Langue|Idioma/).selectOption({ label });
    await expect(page.locator('html')).toHaveAttribute('lang', language);
    for (const [name, hash] of pages) {
      await page.goto(`/${hash}`);
      await expect(page.locator('h1')).toBeVisible();
      await page.waitForLoadState('networkidle');
      await expectAccessible(page, `${name} (${language})`);
    }
  }
  // Back to English for the remaining tests.
  await page.getByLabel(/Language|Langue|Idioma/).selectOption({ label: 'English' });

  // Pages seen before sign-in, with the language taken from the browser.
  for (const [language] of LANGUAGES) {
    const context = await browser.newContext({ locale: language });
    const anonymous = await context.newPage();
    for (const [name, url] of [
      ['sign-in', '/#/login'],
      ['invitation link', pendingInvitation],
      ['not initialized', process.env.HUB_EMPTY_URL],
    ]) {
      await anonymous.goto(url);
      await expect(anonymous.locator('html')).toHaveAttribute('lang', language);
      await expect(anonymous.locator('h1')).toBeVisible();
      await anonymous.waitForLoadState('networkidle');
      await expectAccessible(anonymous, `${name} (${language})`);
    }
    await context.close();
  }

  // The enrollment step, switching language on the page: the sign-in and link endpoints
  // allow 20 requests a minute from one address.
  const context = await browser.newContext();
  const anonymous = await context.newPage();
  await anonymous.goto(pendingInvitation);
  await anonymous.getByRole('button', { name: 'Set up the authenticator' }).click();
  await expect(anonymous.getByTestId('totp-secret')).toBeVisible();
  for (const [language, label] of LANGUAGES) {
    await anonymous.getByLabel(/Language|Langue|Idioma/).selectOption({ label });
    await expect(anonymous.locator('html')).toHaveAttribute('lang', language);
    await expectAccessible(anonymous, `enrollment (${language})`);
  }
  await context.close();
});

test('inactivity signs the console out', async () => {
  const page = ownerPage;
  await page.getByRole('link', { name: 'Instance' }).click();
  // The default limit is 15 minutes; the page clock is moved forward instead of waiting.
  await page.clock.install();
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Instance', level: 1 })).toBeVisible();
  await page.waitForLoadState('networkidle');
  await page.clock.fastForward('14:05');
  await expect(page.getByRole('alertdialog')).toBeVisible();
  await expectAccessible(page, 'inactivity warning');
  await page.clock.fastForward('01:00');
  await expect(page.getByRole('heading', { name: 'Sign in' })).toBeVisible();
  await expect(page.getByText('You were signed out after a period of inactivity.')).toBeVisible();
});

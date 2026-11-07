// Shared steps: authenticator codes, the axe scan, and sign-in.
const crypto = require('node:crypto');
const AxeBuilder = require('@axe-core/playwright').default;
const { expect } = require('@playwright/test');

function base32(text) {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
  let bits = '';
  for (const c of text.replace(/=+$/, '').toUpperCase()) {
    bits += alphabet.indexOf(c).toString(2).padStart(5, '0');
  }
  const bytes = [];
  for (let i = 0; i + 8 <= bits.length; i += 8) bytes.push(parseInt(bits.slice(i, i + 8), 2));
  return Buffer.from(bytes);
}

function codeAt(secret, step) {
  const counter = Buffer.alloc(8);
  counter.writeBigInt64BE(BigInt(step));
  const digest = crypto.createHmac('sha1', base32(secret)).update(counter).digest();
  const offset = digest[digest.length - 1] & 0x0f;
  const value = digest.readUInt32BE(offset) & 0x7fffffff;
  return String(value % 1_000_000).padStart(6, '0');
}

/** An authenticator: each code is used once, as the Hub refuses replays. */
class Authenticator {
  constructor(secret) {
    this.secret = secret;
    this.last = 0;
  }

  async next() {
    for (;;) {
      const current = Math.floor(Date.now() / 30_000);
      const step = Math.max(current, this.last + 1);
      if (step <= current + 1) {
        this.last = step;
        return codeAt(this.secret, step);
      }
      await new Promise((r) => setTimeout(r, 500));
    }
  }
}

/** Scans the page with axe (WCAG 2.2 A and AA) and fails on any violation. */
async function expectAccessible(page, label) {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
    .analyze();
  const summary = results.violations.map(
    (v) => `${v.id}: ${v.help} (${v.nodes.map((n) => n.target.join(' ')).join(', ')})`,
  );
  expect(summary, `accessibility of ${label}`).toEqual([]);
}

/** Completes a setup or invitation link and returns the authenticator. */
async function completeLink(page, link, name, password) {
  await page.goto(link);
  await page.getByRole('button', { name: 'Set up the authenticator' }).click();
  const secret = (await page.getByTestId('totp-secret').textContent()).trim();
  const authenticator = new Authenticator(secret);
  await page.getByLabel('Display name').fill(name);
  await page.getByLabel('New password', { exact: true }).fill(password);
  await page.getByLabel('Repeat the new password').fill(password);
  await page.getByLabel('Authenticator code').fill(await authenticator.next());
  await page.getByRole('button', { name: 'Save and sign in' }).click();
  await expect(page.getByRole('heading', { name: 'Recovery codes' })).toBeVisible();
  await page.getByRole('button', { name: 'I saved them, continue' }).click();
  await expect(page.getByRole('heading', { name: 'Organizations', level: 1 })).toBeVisible();
  return authenticator;
}

module.exports = { Authenticator, codeAt, completeLink, expectAccessible };

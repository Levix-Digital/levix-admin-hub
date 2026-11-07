// The Hub is started by run.sh, which passes its address and setup link in the environment.
const { defineConfig } = require('@playwright/test');

module.exports = defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 180_000,
  reporter: [['list']],
  use: {
    baseURL: process.env.HUB_URL,
    ignoreHTTPSErrors: true,
    locale: 'en-US',
  },
});

/**
 * Minimal Playwright config to test if config is causing the hang
 */
import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './tests',
  use: {
    baseURL: 'http://127.0.0.1:35173',
  },
  projects: [{ name: 'chromium', use: {} }],
})

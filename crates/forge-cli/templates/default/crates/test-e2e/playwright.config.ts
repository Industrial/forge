import { defineConfig, devices } from '@playwright/test'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
/** Project root (crates/test-e2e -> crates -> root). */
const repoRoot = path.resolve(__dirname, '../..')

/** Frontend URL. Required; set by package.json scripts or caller (e.g. bin/test-e2e). */
const baseURL = process.env.E2E_BASE_URL
if (!baseURL) {
  throw new Error(
    'E2E_BASE_URL is required. Run via "bun run test" or set E2E_BASE_URL and E2E_API_URL.',
  )
}

/** API server URL. Required; set by package.json scripts or caller. */
export const API_BASE_URL = process.env.E2E_API_URL
if (!API_BASE_URL) {
  throw new Error(
    'E2E_API_URL is required. Run via "bun run test" or set E2E_BASE_URL and E2E_API_URL.',
  )
}

export default defineConfig({
  testDir: path.join(__dirname, 'tests'),
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: undefined, // use default (CPU cores) for parallel runs
  reporter: [['list']],
  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  timeout: 30_000,
})

export const REPO_ROOT = repoRoot
/** Project under test (when run from project root via bin/test-e2e, this is the project; in monorepo it is .tmp/e2e_prebuilt). */
export const PREBUILT_ROOT = repoRoot

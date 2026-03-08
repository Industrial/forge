import { defineConfig, devices } from '@playwright/test'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
/** Project root (crates/test-e2e -> crates -> root). */
const repoRoot = path.resolve(__dirname, '../..')

/** Non-standard port (like integration tests) to avoid clashing with dev servers. Set by bin/test-e2e. */
const baseURL = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:35173'
/** API server URL for /healthz, etc. (Vite proxies /api and /ws only). Non-standard port; set by bin/test-e2e. */
export const API_BASE_URL = process.env.E2E_API_URL ?? 'http://127.0.0.1:30999'

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

import { defineConfig, devices } from '@playwright/test'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
/** Project root (crates/test-e2e -> crates -> root). */
const repoRoot = path.resolve(__dirname, '../..')

/** Frontend URL from FORGE_FRONTEND_HOST and FORGE_FRONTEND_PORT. Set by bin/test-e2e or caller. */
const frontendHost = process.env.FORGE_FRONTEND_HOST || '127.0.0.1'
const frontendPort = process.env.FORGE_FRONTEND_PORT
if (!frontendPort) {
  throw new Error(
    'FORGE_FRONTEND_PORT is required. Run via "bun run test" or set FORGE_FRONTEND_PORT and FORGE_SERVER_PORT.',
  )
}
const baseURL = `http://${frontendHost}:${frontendPort}`

/** API server URL from FORGE_BACKEND_HOST and FORGE_SERVER_PORT. Set by bin/test-e2e or caller. */
const backendHost = process.env.FORGE_BACKEND_HOST || '127.0.0.1'
const backendPort = process.env.FORGE_SERVER_PORT || process.env.FORGE_BACKEND_PORT
if (!backendPort) {
  throw new Error(
    'FORGE_SERVER_PORT or FORGE_BACKEND_PORT is required. Run via "bun run test" or set FORGE_* env vars.',
  )
}
export const API_BASE_URL = `http://${backendHost}:${backendPort}`

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
  timeout: 60_000,
})

export const REPO_ROOT = repoRoot
/** Project under test (when run from project root via bin/test-e2e, this is the project; in monorepo it is .tmp/e2e_prebuilt). */
export const PREBUILT_ROOT = repoRoot

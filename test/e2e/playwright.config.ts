import { defineConfig, devices } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../..");

const baseURL = process.env.E2E_BASE_URL ?? "http://127.0.0.1:5173";
/** API server URL for /healthz, etc. (Vite proxies /api and /ws only). Set by bin/test-e2e. */
export const API_BASE_URL = process.env.E2E_API_URL ?? "http://127.0.0.1:30999";

export default defineConfig({
	testDir: path.join(__dirname, "tests"),
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: undefined, // use default (CPU cores) for parallel runs
	reporter: [["list"]],
	use: {
		baseURL,
		trace: "on-first-retry",
		screenshot: "only-on-failure",
		video: "retain-on-failure",
	},
	projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
	timeout: 30_000,
});

export const REPO_ROOT = repoRoot;
export const PREBUILT_ROOT = path.join(repoRoot, ".tmp", "e2e_prebuilt");

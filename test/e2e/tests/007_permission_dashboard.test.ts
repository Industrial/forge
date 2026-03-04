import { test, expect } from "@playwright/test";
import {
	prebuiltExists,
	assertProjectLayout,
	assertAuthLayout,
} from "../helpers/prebuilt.js";

const SEED_PASSWORD = "password";

test.describe("e2e permission-based dashboard", () => {
	test.beforeEach(() => {
		expect(prebuiltExists(), "run bin/test-e2e first").toBe(true);
		assertProjectLayout();
		assertAuthLayout();
	});

	test("admin sees all nav items and can open Organizations, Users, Permissions", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("admin@admin.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/dashboard/);

		await expect(page.getByRole("link", { name: "Dashboard" })).toBeVisible();
		await expect(
			page.getByRole("link", { name: "Organizations" }),
		).toBeVisible();
		await expect(page.getByRole("link", { name: "Users" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Permissions" })).toBeVisible();

		await page.getByRole("link", { name: "Organizations" }).click();
		await expect(page).toHaveURL(/\/dashboard\/organizations/);

		await page.getByRole("link", { name: "Users" }).click();
		await expect(page).toHaveURL(/\/dashboard\/users/);

		await page.getByRole("link", { name: "Permissions" }).click();
		await expect(page).toHaveURL(/\/dashboard\/roles-and-permissions/);
	});

	test("org admin has no Organizations nav and is redirected from /dashboard/organizations", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("orgadmin@default.org");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/dashboard/);

		await expect(
			page.getByRole("link", { name: "Organizations" }),
		).not.toBeVisible();
		await expect(page.getByRole("link", { name: "Users" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Permissions" })).toBeVisible();

		await page.goto("/dashboard/organizations");
		await expect(page).toHaveURL(/\/dashboard$/, { timeout: 10_000 });
	});

	test("viewer has Dashboard, Users, and Permissions (read) nav; no Organizations", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("viewer@default.org");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/dashboard/);

		await expect(page.getByRole("link", { name: "Dashboard" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Users" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Permissions" })).toBeVisible();
		await expect(
			page.getByRole("link", { name: "Organizations" }),
		).not.toBeVisible();

		await page.goto("/dashboard/organizations");
		await expect(page).toHaveURL(/\/dashboard$/, { timeout: 10_000 });

		await page.goto("/dashboard/roles-and-permissions");
		await expect(page).toHaveURL(/\/dashboard\/roles-and-permissions/);
	});

	test("admin can load Permissions page and see assignments", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("admin@admin.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/dashboard/);

		await page.getByRole("link", { name: "Permissions" }).click();
		await expect(page).toHaveURL(/\/dashboard\/roles-and-permissions/);
		await expect(
			page.getByRole("heading", { name: "Permissions" }),
		).toBeVisible();
		// After seeds, table or "No assignments" or description should be present (multiple elements may match)
		await expect(
			page
				.getByText(/View and manage role–permission|No assignments|Scope/)
				.first(),
		).toBeVisible();
	});

	test("admin sees users from all orgs on Users page (global-scope)", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("admin@admin.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/dashboard/);

		await page.getByRole("link", { name: "Users" }).click();
		await expect(page).toHaveURL(/\/dashboard\/users/);
		await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
		// Global-scope admin sees users from all orgs; seeds include "Other" org members (multiple cells contain "Other")
		await expect(
			page
				.getByRole("cell", { name: /Other: (owner|admin|viewer|editor)/ })
				.first(),
		).toBeVisible();
	});
});

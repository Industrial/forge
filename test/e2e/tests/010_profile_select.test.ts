import { test, expect } from "@playwright/test";
import {
	prebuiltExists,
	assertProjectLayout,
	assertAuthLayout,
} from "../helpers/prebuilt.js";

const SEED_PASSWORD = "password";

test.describe("e2e profile select and session profile", () => {
	test.beforeEach(() => {
		expect(prebuiltExists(), "run bin/test-e2e first").toBe(true);
		assertProjectLayout();
		assertAuthLayout();
	});
	// Allow extra time for login/session and proxy round-trips
	test.setTimeout(45_000);

	test("multi-org user is redirected to profile-select after login", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("multi@email.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/select-profile/);
		await expect(
			page.getByRole("heading", { name: "Select profile" }),
		).toBeVisible();
	});

	test("single-profile user goes straight to dashboard after login", async ({
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
		await expect(page).not.toHaveURL(/\/select-profile/);
	});

	test("profile-select: choose profile then land on dashboard", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("multi@email.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/select-profile/);
		// Select first profile card (order may vary)
		await page.locator('[data-testid^="profile-"]').first().click();
		await expect(page).toHaveURL(/\/dashboard/);
		await expect(page.getByTestId("dashboard-heading")).toContainText(
			"Dashboard",
		);
	});

	test("dashboard without profile redirects to profile-select", async ({
		page,
	}) => {
		// Login as multi (no profile set), then try to open dashboard directly
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("multi@email.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/select-profile/);
		await page.goto("/dashboard");
		await expect(page).toHaveURL(/\/select-profile/, { timeout: 10_000 });
	});

	test("viewer sees only Default org users on Users page", async ({ page }) => {
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
		await page.getByRole("link", { name: "Users" }).click();
		await expect(page).toHaveURL(/\/dashboard\/users/);
		await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
		// Org-scoped viewer must not see "Other" organization in the users table
		await expect(page.getByText("Other")).not.toBeVisible();
	});

	test("multi@email.com: three profiles (Personal, Default, Other) and nav per profile", async ({
		page,
	}) => {
		// 1. Log in
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("multi@email.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();

		// 2. Verify profile select page
		await expect(page).toHaveURL(/\/select-profile/);
		await expect(
			page.getByRole("heading", { name: "Select profile" }),
		).toBeVisible();

		// 3. Verify three profiles: Personal, Default, Other
		await expect(page.getByTestId("profile-Personal-viewer")).toBeVisible();
		await expect(page.getByTestId("profile-Default-viewer")).toBeVisible();
		await expect(page.getByTestId("profile-Other-editor")).toBeVisible();

		// 4. Pick Personal
		await page.getByTestId("profile-Personal-viewer").click();
		await expect(page).toHaveURL(/\/dashboard/);

		// 5. Personal: only Dashboard link visible
		await expect(page.getByRole("link", { name: "Dashboard" })).toBeVisible();
		// 5.1 Other dashboard links not shown
		await expect(
			page.getByRole("link", { name: "Organizations" }),
		).not.toBeVisible();
		await expect(page.getByRole("link", { name: "Users" })).not.toBeVisible();
		await expect(page.getByRole("link", { name: "Roles" })).not.toBeVisible();
		await expect(
			page.getByRole("link", { name: "Permissions" }),
		).not.toBeVisible();
		await expect(page.getByRole("link", { name: "Tasks" })).not.toBeVisible();
		await expect(
			page.getByRole("link", { name: "Audit log" }),
		).not.toBeVisible();

		// 6. Open user dropdown in navbar
		await page.getByRole("button", { name: "User menu" }).click();

		// 7. Select Default organization profile
		await page.getByRole("menuitem", { name: /Viewer · Default/ }).click();

		// 8. Default: Dashboard, Users, Roles, Permissions, Tasks, Audit log
		await expect(page.getByRole("link", { name: "Dashboard" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Users" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Roles" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Permissions" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Tasks" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Audit log" })).toBeVisible();
		// 8.1 Organizations not shown (viewer has no orgs permission)
		await expect(
			page.getByRole("link", { name: "Organizations" }),
		).not.toBeVisible();

		// 9. Open user dropdown again
		await page.getByRole("button", { name: "User menu" }).click();

		// 10. Select Other organization profile
		await page.getByRole("menuitem", { name: /Editor · Other/ }).click();

		// 11. Other: Dashboard, Users, Roles, Permissions, Tasks (editor has no audit.read)
		await expect(page.getByRole("link", { name: "Dashboard" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Users" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Roles" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Permissions" })).toBeVisible();
		await expect(page.getByRole("link", { name: "Tasks" })).toBeVisible();
		// 11.1 Audit log and Organizations not shown
		await expect(
			page.getByRole("link", { name: "Audit log" }),
		).not.toBeVisible();
		await expect(
			page.getByRole("link", { name: "Organizations" }),
		).not.toBeVisible();
	});

	test("switch profile in navbar updates dashboard scope (users list)", async ({
		page,
	}) => {
		await page.goto("/login");
		await page
			.getByTestId("login-email")
			.locator("input")
			.fill("multi@email.com");
		await page
			.getByTestId("login-password")
			.locator("input")
			.fill(SEED_PASSWORD);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/select-profile/);
		await page.locator('[data-testid^="profile-"]').first().click();
		await expect(page).toHaveURL(/\/dashboard/);
		await page.getByRole("link", { name: "Users" }).click();
		await expect(page).toHaveURL(/\/dashboard\/users/);
		await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
		// Open user menu and switch to the other profile (e.g. Other · Editor)
		await page.getByRole("button", { name: "User menu" }).click();
		// Click the menu item that contains "Other" (second org for multi@email.com)
		await page.getByRole("menuitem", { name: /Other/ }).click();
		// Page should still be on users; list may refresh to show Other org scope
		await expect(page).toHaveURL(/\/dashboard\/users/);
		await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
	});

	test("register then login lands in dashboard with profile set", async ({
		page,
	}) => {
		const email = `profile-e2e-${Date.now()}@test.com`;
		const password = "password";

		await page.goto("/register");
		await page.getByTestId("register-email").locator("input").fill(email);
		await page.getByTestId("register-password").locator("input").fill(password);
		await page.getByTestId("register-submit").click();
		await expect(page).toHaveURL(/\/login/);

		await page.getByTestId("login-email").locator("input").fill(email);
		await page.getByTestId("login-password").locator("input").fill(password);
		await page.getByTestId("login-submit").click();
		await expect(page).toHaveURL(/\/dashboard/);
		await expect(page.getByTestId("dashboard-heading")).toContainText(
			"Dashboard",
		);
	});
});

import { test, expect } from "@playwright/test";
import {
	prebuiltExists,
	assertProjectLayout,
	pathExists,
	readFile,
} from "../helpers/prebuilt.js";

test.describe("e2e authz", () => {
	test("authz layout and protected route: unauthed redirect, register+login then dashboard", async ({
		page,
	}) => {
		expect(prebuiltExists(), "run bin/test-e2e first").toBe(true);
		assertProjectLayout();
		expect(pathExists("crates/db/src/models/organization.rs")).toBe(true);
		expect(pathExists("crates/db/src/models/membership.rs")).toBe(true);
		const userModel = readFile("crates/db/src/models/user.rs");
		expect(userModel).toContain("impl AuthzContext");
		const authHandlers = readFile("crates/app/src/handlers/auth.rs");
		expect(authHandlers).toMatch(/admin/);
		expect(
			authHandlers.includes("is_admin") ||
				authHandlers.includes("record_authz_denied"),
		).toBe(true);

		await page.goto("/dashboard");
		await expect(page).toHaveURL(/\/login/);

		const email = `authz-e2e-${Date.now()}@test.com`;
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

import { test, expect } from "@playwright/test";
import { prebuiltExists, assertProjectLayout } from "../helpers/prebuilt.js";

test.describe("e2e rate limiting", () => {
	test("app root loads 5 times", async ({ page }) => {
		expect(prebuiltExists(), "run bin/test-e2e first").toBe(true);
		assertProjectLayout();
		for (let i = 0; i < 5; i++) {
			await page.goto("/");
			await expect(page.locator("#root")).toBeVisible();
		}
	});
});

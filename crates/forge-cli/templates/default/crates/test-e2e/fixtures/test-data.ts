/**
 * Test data factories using Effect.ts
 * Generate unique data per test run to avoid collisions in parallel execution
 */

export type TestUser = {
  email: string
  password: string
  name?: string
}

export type TestOrganization = {
  name: string
  slug: string
}

/**
 * Create a unique test user with timestamp and random suffix
 * Ensures no collisions in parallel test execution
 */
export const createTestUser = (overrides?: Partial<TestUser>): TestUser => {
  const timestamp = Date.now()
  const random = Math.random().toString(36).substring(7)

  return {
    email: `test-${timestamp}-${random}@example.com`,
    password: 'TestPassword123!',
    name: `Test User ${timestamp}`,
    ...overrides,
  }
}

/**
 * Create a unique test organization with timestamp and random suffix
 */
export const createTestOrganization = (
  overrides?: Partial<TestOrganization>,
): TestOrganization => {
  const timestamp = Date.now()
  const random = Math.random().toString(36).substring(7)

  return {
    name: `Test Org ${timestamp}`,
    slug: `test-org-${timestamp}-${random}`,
    ...overrides,
  }
}

/**
 * Seed data users (reused across tests)
 * These are created by the seed script and reused for authenticated tests
 */
export const SEED_USERS = {
  viewer: { email: 'viewer@default.org', password: 'password' },
  editor: { email: 'editor@default.org', password: 'password' },
  orgOwner: { email: 'owner@default.org', password: 'password' },
  orgAdmin: { email: 'orgadmin@default.org', password: 'password' },
  appAdmin: { email: 'admin@admin.com', password: 'password' },
  multiProfile: { email: 'multi@email.com', password: 'password' },
} as const

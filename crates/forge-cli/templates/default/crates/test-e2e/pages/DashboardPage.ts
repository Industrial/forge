import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'

/**
 * DashboardPage service - Effect.ts service for dashboard page interactions
 * Methods take PlaywrightPage from context and return Effects
 */
export class DashboardPage extends Context.Tag('DashboardPage')<
  DashboardPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly sidebar: () => Effect.Effect<Locator>
    readonly usersLink: () => Effect.Effect<Locator>
    readonly rolesLink: () => Effect.Effect<Locator>
    readonly clickUsersLink: () => Effect.Effect<void>
    readonly clickRolesLink: () => Effect.Effect<void>
  }
>() {}

/**
 * Create DashboardPage layer from PlaywrightPage
 */
export const DashboardPageLive = Layer.effect(
  DashboardPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('dashboard-page')
        }),

      sidebar: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('dashboard-sidebar')
        }),

      usersLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-users-link')
        }),

      rolesLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-roles-link')
        }),

      clickUsersLink: () =>
        Effect.gen(function* () {
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('sidebar-users-link').click()
          )
        }),

      clickRolesLink: () =>
        Effect.gen(function* () {
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('sidebar-roles-link').click()
          )
        }),
    }
  })
)

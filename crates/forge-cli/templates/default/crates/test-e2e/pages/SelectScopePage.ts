import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'

/**
 * SelectScopePage service - Effect.ts service for scope selection page interactions
 * Methods take PlaywrightPage from context and return Effects
 */
export class SelectScopePage extends Context.Tag('SelectScopePage')<
  SelectScopePage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly profileList: () => Effect.Effect<Locator>
    readonly profileCard: (index: number) => Effect.Effect<Locator>
    readonly profileSelectButton: (index: number) => Effect.Effect<Locator>
    readonly selectProfile: (index: number) => Effect.Effect<void>
  }
>() {}

/**
 * Create SelectScopePage layer from PlaywrightPage
 */
export const SelectScopePageLive = Layer.effect(
  SelectScopePage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('select-scope-page')
        }),

      profileList: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('profile-list')
        }),

      profileCard: (index: number) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`profile-card-${index}`)
        }),

      profileSelectButton: (index: number) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`profile-select-button-${index}`)
        }),

      selectProfile: (index: number) =>
        Effect.gen(function* () {
          yield* Effect.promise(() =>
            playwrightPage.getByTestId(`profile-select-button-${index}`).click()
          )
          yield* Effect.promise(() =>
            playwrightPage.waitForURL('/dashboard', { timeout: 5000 })
          )
        }),
    }
  })
)

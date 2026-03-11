import type { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

/**
 * ProfilePage service - Effect.ts service for profile/scope selection page interactions
 */
export class ProfilePage extends Context.Tag('ProfilePage')<
  ProfilePage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly pageTitle: () => Effect.Effect<Locator>
    readonly currentProfile: () => Effect.Effect<Locator>
    readonly currentProfileOrgName: () => Effect.Effect<Locator>
    readonly currentProfileRoleName: () => Effect.Effect<Locator>
    readonly profileList: () => Effect.Effect<Locator>
    readonly profileCard: (index: number) => Effect.Effect<Locator>
    readonly profileSwitchButton: (index: number) => Effect.Effect<Locator>
    readonly profileDetails: () => Effect.Effect<Locator>
    readonly switchProfile: (index: number) => Effect.Effect<void>
  }
>() {}

export const ProfilePageLive = Layer.effect(
  ProfilePage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('profile-page')
        }),

      pageTitle: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('profile-page-title')
        }),

      currentProfile: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('current-profile')
        }),

      currentProfileOrgName: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('current-profile-org-name')
        }),

      currentProfileRoleName: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('current-profile-role-name')
        }),

      profileList: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('profile-list')
        }),

      profileCard: (index: number) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`profile-card-${index}`)
        }),

      profileSwitchButton: (index: number) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`profile-switch-button-${index}`)
        }),

      profileDetails: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('profile-details')
        }),

      switchProfile: (index: number) =>
        Effect.gen(function* () {
          const profileCard = playwrightPage.getByTestId(
            `profile-card-${index}`,
          )
          yield* LocatorHelpers.click(profileCard)
          yield* PageHelpers.waitForURL(playwrightPage, '/dashboard', {
            timeout: 5000,
          })
        }),
    }
  }),
)

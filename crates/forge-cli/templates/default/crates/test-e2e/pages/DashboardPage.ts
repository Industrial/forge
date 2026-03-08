import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

/**
 * DashboardPage service - Effect.ts service for dashboard page interactions
 * Methods take PlaywrightPage from context and return Effects
 */
export class DashboardPage extends Context.Tag('DashboardPage')<
  DashboardPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly layout: () => Effect.Effect<Locator>
    readonly sidebar: () => Effect.Effect<Locator>
    readonly navbar: () => Effect.Effect<Locator>
    readonly content: () => Effect.Effect<Locator>
    readonly dashboardLink: () => Effect.Effect<Locator>
    readonly usersLink: () => Effect.Effect<Locator>
    readonly rolesLink: () => Effect.Effect<Locator>
    readonly permissionsLink: () => Effect.Effect<Locator>
    readonly auditLogLink: () => Effect.Effect<Locator>
    readonly organizationsLink: () => Effect.Effect<Locator>
    readonly themeToggleButton: () => Effect.Effect<Locator>
    readonly userMenu: () => Effect.Effect<Locator>
    readonly userMenuDropdown: () => Effect.Effect<Locator>
    readonly logoutButton: () => Effect.Effect<Locator>
    readonly clickUsersLink: () => Effect.Effect<void>
    readonly clickRolesLink: () => Effect.Effect<void>
    readonly clickPermissionsLink: () => Effect.Effect<void>
    readonly clickAuditLogLink: () => Effect.Effect<void>
    readonly clickOrganizationsLink: () => Effect.Effect<void>
    readonly toggleTheme: () => Effect.Effect<void>
    readonly openUserMenu: () => Effect.Effect<void>
    readonly logout: () => Effect.Effect<void>
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

      layout: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('dashboard-layout')
        }),

      sidebar: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('dashboard-sidebar')
        }),

      navbar: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('dashboard-navbar')
        }),

      content: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('dashboard-content')
        }),

      dashboardLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-dashboard-link')
        }),

      usersLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-users-link')
        }),

      rolesLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-roles-link')
        }),

      permissionsLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-permissions-link')
        }),

      auditLogLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-audit-log-link')
        }),

      organizationsLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('sidebar-organizations-link')
        }),

      themeToggleButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('theme-toggle-button')
        }),

      userMenu: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('navbar-user-menu')
        }),

      userMenuDropdown: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-menu-dropdown')
        }),

      logoutButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-menu-logout-button')
        }),

      clickUsersLink: () =>
        Effect.gen(function* () {
          const usersLink = playwrightPage.getByTestId('sidebar-users-link')
          yield* LocatorHelpers.click(usersLink)
        }),

      clickRolesLink: () =>
        Effect.gen(function* () {
          const rolesLink = playwrightPage.getByTestId('sidebar-roles-link')
          yield* LocatorHelpers.click(rolesLink)
        }),

      clickPermissionsLink: () =>
        Effect.gen(function* () {
          const permissionsLink = playwrightPage.getByTestId(
            'sidebar-permissions-link',
          )
          yield* LocatorHelpers.click(permissionsLink)
        }),

      clickAuditLogLink: () =>
        Effect.gen(function* () {
          const auditLogLink = playwrightPage.getByTestId(
            'sidebar-audit-log-link',
          )
          yield* LocatorHelpers.click(auditLogLink)
        }),

      clickOrganizationsLink: () =>
        Effect.gen(function* () {
          const organizationsLink = playwrightPage.getByTestId(
            'sidebar-organizations-link',
          )
          yield* LocatorHelpers.click(organizationsLink)
        }),

      toggleTheme: () =>
        Effect.gen(function* () {
          const themeToggle = playwrightPage.getByTestId('theme-toggle-button')
          yield* LocatorHelpers.click(themeToggle)
        }),

      openUserMenu: () =>
        Effect.gen(function* () {
          const userMenu = playwrightPage.getByTestId('navbar-user-menu')
          yield* LocatorHelpers.click(userMenu)
        }),

      logout: () =>
        Effect.gen(function* () {
          const userMenu = playwrightPage.getByTestId('navbar-user-menu')
          const logoutButton = playwrightPage.getByTestId(
            'user-menu-logout-button',
          )

          yield* LocatorHelpers.click(userMenu)
          yield* LocatorHelpers.click(logoutButton)
          yield* PageHelpers.waitForURL(
            playwrightPage,
            '/authentication/login',
            { timeout: 5000 },
          )
        }),
    }
  }),
)

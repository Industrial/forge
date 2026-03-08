import { Page } from '@playwright/test'
import { Layer } from 'effect'
import {
  LoginPage,
  LoginPageLive,
  RegisterPage,
  RegisterPageLive,
  DashboardPage,
  DashboardPageLive,
  SelectScopePage,
  SelectScopePageLive,
  UsersPage,
  UsersPageLive,
  RolesPage,
  RolesPageLive,
  PermissionsPage,
  PermissionsPageLive,
  AuditLogPage,
  AuditLogPageLive,
  OrganizationsPage,
  OrganizationsPageLive,
  ProfilePage,
  ProfilePageLive,
} from '@/pages'
import { createPlaywrightPageLayer, PlaywrightPage } from '@/fixtures/playwright'

/**
 * Create all page layers from a Playwright Page instance
 * Combines PlaywrightPage with all page services
 * Page services depend on PlaywrightPage, so we provide it to them
 */
export const createPageLayers = (page: Page) => {
  const playwrightPageLayer = createPlaywrightPageLayer(page)
  
  return Layer.mergeAll(
    playwrightPageLayer,
    LoginPageLive.pipe(Layer.provide(playwrightPageLayer)),
    RegisterPageLive.pipe(Layer.provide(playwrightPageLayer)),
    DashboardPageLive.pipe(Layer.provide(playwrightPageLayer)),
    SelectScopePageLive.pipe(Layer.provide(playwrightPageLayer)),
    UsersPageLive.pipe(Layer.provide(playwrightPageLayer)),
    RolesPageLive.pipe(Layer.provide(playwrightPageLayer)),
    PermissionsPageLive.pipe(Layer.provide(playwrightPageLayer)),
    AuditLogPageLive.pipe(Layer.provide(playwrightPageLayer)),
    OrganizationsPageLive.pipe(Layer.provide(playwrightPageLayer)),
    ProfilePageLive.pipe(Layer.provide(playwrightPageLayer)),
  )
}

/**
 * Type for the combined page layers context
 */
export type PageLayersContext =
  | PlaywrightPage
  | LoginPage
  | RegisterPage
  | DashboardPage
  | SelectScopePage
  | UsersPage
  | RolesPage
  | PermissionsPage
  | AuditLogPage
  | OrganizationsPage
  | ProfilePage

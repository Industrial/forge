import type { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '../fixtures/playwright'
import * as LocatorHelpers from '../helpers/locator'

/**
 * PermissionsPage service - Effect.ts service for permissions page interactions
 */
export class PermissionsPage extends Context.Tag('PermissionsPage')<
  PermissionsPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly pageTitle: () => Effect.Effect<Locator>
    readonly list: () => Effect.Effect<Locator>
    readonly table: () => Effect.Effect<Locator>
    readonly addButton: () => Effect.Effect<Locator>
    readonly assignmentRow: (
      roleName: string,
      permissionName: string,
    ) => Effect.Effect<Locator>
    readonly removeButton: (
      roleName: string,
      permissionName: string,
    ) => Effect.Effect<Locator>
    readonly addDialog: () => Effect.Effect<Locator>
    readonly addForm: () => Effect.Effect<Locator>
    readonly addRoleSelect: () => Effect.Effect<Locator>
    readonly addPermissionSelect: () => Effect.Effect<Locator>
    readonly addSubmitButton: () => Effect.Effect<Locator>
    readonly removeConfirmDialog: () => Effect.Effect<Locator>
    readonly removeConfirmButton: () => Effect.Effect<Locator>
    readonly addPermission: (
      roleName: string,
      permissionName: string,
    ) => Effect.Effect<void>
    readonly removePermission: (
      roleName: string,
      permissionName: string,
    ) => Effect.Effect<void>
  }
>() {}

export const PermissionsPageLive = Layer.effect(
  PermissionsPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permissions-page')
        }),

      pageTitle: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permissions-page-title')
        }),

      list: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permissions-list')
        }),

      table: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permissions-table')
        }),

      addButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-add-button')
        }),

      assignmentRow: (roleName: string, permissionName: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(
            `permission-assignment-row-${roleName}-${permissionName}`,
          )
        }),

      removeButton: (roleName: string, permissionName: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(
            `permission-remove-button-${roleName}-${permissionName}`,
          )
        }),

      addDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-add-dialog')
        }),

      addForm: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-add-form')
        }),

      addRoleSelect: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-add-role-select')
        }),

      addPermissionSelect: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-add-permission-select')
        }),

      addSubmitButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-add-submit-button')
        }),

      removeConfirmDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-remove-confirm-dialog')
        }),

      removeConfirmButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('permission-remove-confirm-button')
        }),

      addPermission: (roleName: string, permissionName: string) =>
        Effect.gen(function* () {
          const addButton = playwrightPage.getByTestId('permission-add-button')
          const addDialog = playwrightPage.getByTestId('permission-add-dialog')
          const roleSelect = playwrightPage.getByTestId(
            'permission-add-role-select',
          )
          const permissionSelect = playwrightPage.getByTestId(
            'permission-add-permission-select',
          )
          const submitButton = playwrightPage.getByTestId(
            'permission-add-submit-button',
          )

          yield* LocatorHelpers.click(addButton)
          yield* LocatorHelpers.waitForVisible(addDialog)
          yield* LocatorHelpers.selectOption(roleSelect, roleName)
          yield* LocatorHelpers.selectOption(permissionSelect, permissionName)
          yield* LocatorHelpers.click(submitButton)
          yield* LocatorHelpers.waitForHidden(addDialog)
        }),

      removePermission: (roleName: string, permissionName: string) =>
        Effect.gen(function* () {
          const removeButton = playwrightPage.getByTestId(
            `permission-remove-button-${roleName}-${permissionName}`,
          )
          const confirmDialog = playwrightPage.getByTestId(
            'permission-remove-confirm-dialog',
          )
          const confirmButton = playwrightPage.getByTestId(
            'permission-remove-confirm-button',
          )

          yield* LocatorHelpers.click(removeButton)
          yield* LocatorHelpers.waitForVisible(confirmDialog)
          yield* LocatorHelpers.click(confirmButton)
          yield* LocatorHelpers.waitForHidden(confirmDialog)
        }),
    }
  }),
)

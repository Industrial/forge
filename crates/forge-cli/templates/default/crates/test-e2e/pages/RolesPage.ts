import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'

/**
 * RolesPage service - Effect.ts service for roles page interactions
 */
export class RolesPage extends Context.Tag('RolesPage')<
  RolesPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly pageTitle: () => Effect.Effect<Locator>
    readonly list: () => Effect.Effect<Locator>
    readonly table: () => Effect.Effect<Locator>
    readonly createButton: () => Effect.Effect<Locator>
    readonly row: (name: string) => Effect.Effect<Locator>
    readonly viewButton: (name: string) => Effect.Effect<Locator>
    readonly editButton: (name: string) => Effect.Effect<Locator>
    readonly deleteButton: (name: string) => Effect.Effect<Locator>
    readonly detailsDialog: () => Effect.Effect<Locator>
    readonly createDialog: () => Effect.Effect<Locator>
    readonly createForm: () => Effect.Effect<Locator>
    readonly createNameInput: () => Effect.Effect<Locator>
    readonly createSubmitButton: () => Effect.Effect<Locator>
    readonly editDialog: () => Effect.Effect<Locator>
    readonly editForm: () => Effect.Effect<Locator>
    readonly editNameInput: () => Effect.Effect<Locator>
    readonly editSubmitButton: () => Effect.Effect<Locator>
    readonly deleteConfirmDialog: () => Effect.Effect<Locator>
    readonly deleteConfirmButton: () => Effect.Effect<Locator>
    readonly createRole: (name: string) => Effect.Effect<void>
    readonly updateRole: (name: string, newName: string) => Effect.Effect<void>
    readonly deleteRole: (name: string) => Effect.Effect<void>
    readonly viewRoleDetails: (name: string) => Effect.Effect<void>
  }
>() {}

export const RolesPageLive = Layer.effect(
  RolesPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('roles-page')
        }),

      pageTitle: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('roles-page-title')
        }),

      list: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('roles-list')
        }),

      table: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('roles-table')
        }),

      createButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('roles-create-button')
        }),

      row: (name: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`role-row-${name}`)
        }),

      viewButton: (name: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`role-view-button-${name}`)
        }),

      editButton: (name: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`role-edit-button-${name}`)
        }),

      deleteButton: (name: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`role-delete-button-${name}`)
        }),

      detailsDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-details-dialog')
        }),

      createDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-create-dialog')
        }),

      createForm: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-create-form')
        }),

      createNameInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-create-name-input')
        }),

      createSubmitButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-create-submit-button')
        }),

      editDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-edit-dialog')
        }),

      editForm: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-edit-form')
        }),

      editNameInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-edit-name-input')
        }),

      editSubmitButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-edit-submit-button')
        }),

      deleteConfirmDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-delete-confirm-dialog')
        }),

      deleteConfirmButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('role-delete-confirm-button')
        }),

      createRole: (name: string) =>
        Effect.gen(function* () {
          const createButton = playwrightPage.getByTestId('roles-create-button')
          const createDialog = playwrightPage.getByTestId('role-create-dialog')
          const nameInput = playwrightPage.getByTestId('role-create-name-input')
          const submitButton = playwrightPage.getByTestId('role-create-submit-button')
          
          yield* LocatorHelpers.click(createButton)
          yield* LocatorHelpers.waitForVisible(createDialog)
          yield* LocatorHelpers.fill(nameInput, name)
          yield* LocatorHelpers.click(submitButton)
          yield* LocatorHelpers.waitForHidden(createDialog)
        }),

      updateRole: (name: string, newName: string) =>
        Effect.gen(function* () {
          const row = playwrightPage.getByTestId(`role-row-${name}`)
          const editDialog = playwrightPage.getByTestId('role-edit-dialog')
          const nameInput = playwrightPage.getByTestId('role-edit-name-input')
          const submitButton = playwrightPage.getByTestId('role-edit-submit-button')
          
          yield* LocatorHelpers.click(row)
          yield* LocatorHelpers.waitForVisible(editDialog)
          yield* LocatorHelpers.fill(nameInput, newName)
          yield* LocatorHelpers.click(submitButton)
          yield* LocatorHelpers.waitForHidden(editDialog)
        }),

      deleteRole: (name: string) =>
        Effect.gen(function* () {
          const deleteButton = playwrightPage.getByTestId(`role-delete-button-${name}`)
          const confirmDialog = playwrightPage.getByTestId('role-delete-confirm-dialog')
          const confirmButton = playwrightPage.getByTestId('role-delete-confirm-button')
          
          yield* LocatorHelpers.click(deleteButton)
          yield* LocatorHelpers.waitForVisible(confirmDialog)
          yield* LocatorHelpers.click(confirmButton)
          yield* LocatorHelpers.waitForHidden(confirmDialog)
        }),

      viewRoleDetails: (name: string) =>
        Effect.gen(function* () {
          const row = playwrightPage.getByTestId(`role-row-${name}`)
          const detailsDialog = playwrightPage.getByTestId('role-details-dialog')
          
          yield* LocatorHelpers.click(row)
          yield* LocatorHelpers.waitForVisible(detailsDialog)
        }),
    }
  })
)

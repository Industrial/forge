import type { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'

/**
 * UsersPage service - Effect.ts service for users page interactions
 */
export class UsersPage extends Context.Tag('UsersPage')<
  UsersPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly pageTitle: () => Effect.Effect<Locator>
    readonly list: () => Effect.Effect<Locator>
    readonly table: () => Effect.Effect<Locator>
    readonly filterInput: () => Effect.Effect<Locator>
    readonly createButton: () => Effect.Effect<Locator>
    readonly row: (email: string) => Effect.Effect<Locator>
    readonly viewButton: (email: string) => Effect.Effect<Locator>
    readonly editButton: (email: string) => Effect.Effect<Locator>
    readonly deleteButton: (email: string) => Effect.Effect<Locator>
    readonly detailsDialog: () => Effect.Effect<Locator>
    readonly createDialog: () => Effect.Effect<Locator>
    readonly createForm: () => Effect.Effect<Locator>
    readonly createEmailInput: () => Effect.Effect<Locator>
    readonly createPasswordInput: () => Effect.Effect<Locator>
    readonly createSubmitButton: () => Effect.Effect<Locator>
    readonly editDialog: () => Effect.Effect<Locator>
    readonly editForm: () => Effect.Effect<Locator>
    readonly editEmailInput: () => Effect.Effect<Locator>
    readonly editSubmitButton: () => Effect.Effect<Locator>
    readonly deleteConfirmDialog: () => Effect.Effect<Locator>
    readonly deleteConfirmButton: () => Effect.Effect<Locator>
    readonly filter: (query: string) => Effect.Effect<void>
    readonly createUser: (
      email: string,
      password: string,
    ) => Effect.Effect<void>
    readonly updateUser: (
      email: string,
      newEmail: string,
    ) => Effect.Effect<void>
    readonly deleteUser: (email: string) => Effect.Effect<void>
    readonly viewUserDetails: (email: string) => Effect.Effect<void>
  }
>() {}

export const UsersPageLive = Layer.effect(
  UsersPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('users-page')
        }),

      pageTitle: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('users-page-title')
        }),

      list: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('users-list')
        }),

      table: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('users-table')
        }),

      filterInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('users-filter-input')
        }),

      createButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('users-create-button')
        }),

      row: (email: string) =>
        Effect.gen(function* () {
          // Find row by email content since testid is generic 'user-row'
          // Use locator with text filter
          return playwrightPage
            .locator('[data-testid="user-row"]')
            .filter({ hasText: email })
        }),

      viewButton: (email: string) =>
        Effect.gen(function* () {
          // Find button within the row containing the email
          const row = yield* Effect.gen(function* () {
            return playwrightPage
              .locator('[data-testid="user-row"]')
              .filter({ hasText: email })
          })
          return row.getByTestId('user-view-button')
        }),

      editButton: (email: string) =>
        Effect.gen(function* () {
          const row = yield* Effect.gen(function* () {
            return playwrightPage
              .locator('[data-testid="user-row"]')
              .filter({ hasText: email })
          })
          return row.getByTestId('user-edit-button')
        }),

      deleteButton: (email: string) =>
        Effect.gen(function* () {
          const row = yield* Effect.gen(function* () {
            return playwrightPage
              .locator('[data-testid="user-row"]')
              .filter({ hasText: email })
          })
          return row.getByTestId('user-delete-button')
        }),

      detailsDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-details-dialog')
        }),

      createDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-create-dialog')
        }),

      createForm: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-create-form')
        }),

      createEmailInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-create-email-input')
        }),

      createPasswordInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-create-password-input')
        }),

      createSubmitButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-create-submit-button')
        }),

      editDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-edit-dialog')
        }),

      editForm: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-edit-form')
        }),

      editEmailInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-edit-email-input')
        }),

      editSubmitButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-edit-submit-button')
        }),

      deleteConfirmDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-delete-confirm-dialog')
        }),

      deleteConfirmButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('user-delete-confirm-button')
        }),

      filter: (query: string) =>
        Effect.gen(function* () {
          const filterInput = playwrightPage.getByTestId('users-filter-input')
          yield* LocatorHelpers.fill(filterInput, query)
        }),

      createUser: (email: string, password: string) =>
        Effect.gen(function* () {
          const createButton = playwrightPage.getByTestId('users-create-button')
          const createDialog = playwrightPage.getByTestId('user-create-dialog')
          const emailInput = playwrightPage.getByTestId(
            'user-create-email-input',
          )
          const passwordInput = playwrightPage.getByTestId(
            'user-create-password-input',
          )
          const submitButton = playwrightPage.getByTestId(
            'user-create-submit-button',
          )

          yield* LocatorHelpers.click(createButton)
          yield* LocatorHelpers.waitForVisible(createDialog)
          yield* LocatorHelpers.fill(emailInput, email)
          yield* LocatorHelpers.fill(passwordInput, password)
          yield* LocatorHelpers.click(submitButton)
          yield* LocatorHelpers.waitForHidden(createDialog)
        }),

      updateUser: (email: string, newEmail: string) =>
        Effect.gen(function* () {
          const row = playwrightPage
            .locator('[data-testid="user-row"]')
            .filter({ hasText: email })
          const editDialog = playwrightPage.getByTestId('user-edit-dialog')
          const emailInput = playwrightPage.getByTestId('user-edit-email-input')
          const submitButton = playwrightPage.getByTestId(
            'user-edit-submit-button',
          )

          yield* LocatorHelpers.click(row)
          yield* LocatorHelpers.waitForVisible(editDialog)
          yield* LocatorHelpers.fill(emailInput, newEmail)
          yield* LocatorHelpers.click(submitButton)
          yield* LocatorHelpers.waitForHidden(editDialog)
        }),

      deleteUser: (email: string) =>
        Effect.gen(function* () {
          const row = playwrightPage
            .locator('[data-testid="user-row"]')
            .filter({ hasText: email })
          const deleteButton = row.getByTestId('user-delete-button')
          const confirmDialog = playwrightPage.getByTestId(
            'user-delete-confirm-dialog',
          )
          const confirmButton = playwrightPage.getByTestId(
            'user-delete-confirm-button',
          )

          yield* LocatorHelpers.click(deleteButton)
          yield* LocatorHelpers.waitForVisible(confirmDialog)
          yield* LocatorHelpers.click(confirmButton)
          yield* LocatorHelpers.waitForHidden(confirmDialog)
        }),

      viewUserDetails: (email: string) =>
        Effect.gen(function* () {
          const row = playwrightPage
            .locator('[data-testid="user-row"]')
            .filter({ hasText: email })
          const detailsDialog = playwrightPage.getByTestId(
            'user-details-dialog',
          )

          yield* LocatorHelpers.click(row)
          yield* LocatorHelpers.waitForVisible(detailsDialog)
        }),
    }
  }),
)

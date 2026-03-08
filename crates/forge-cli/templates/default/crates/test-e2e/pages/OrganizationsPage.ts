import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'

/**
 * OrganizationsPage service - Effect.ts service for organizations page interactions
 */
export class OrganizationsPage extends Context.Tag('OrganizationsPage')<
  OrganizationsPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly pageTitle: () => Effect.Effect<Locator>
    readonly list: () => Effect.Effect<Locator>
    readonly table: () => Effect.Effect<Locator>
    readonly filterInput: () => Effect.Effect<Locator>
    readonly createButton: () => Effect.Effect<Locator>
    readonly row: (name: string) => Effect.Effect<Locator>
    readonly editButton: (name: string) => Effect.Effect<Locator>
    readonly deleteButton: (name: string) => Effect.Effect<Locator>
    readonly createDialog: () => Effect.Effect<Locator>
    readonly createForm: () => Effect.Effect<Locator>
    readonly createNameInput: () => Effect.Effect<Locator>
    readonly createSlugInput: () => Effect.Effect<Locator>
    readonly createSubmitButton: () => Effect.Effect<Locator>
    readonly editDialog: () => Effect.Effect<Locator>
    readonly editForm: () => Effect.Effect<Locator>
    readonly editNameInput: () => Effect.Effect<Locator>
    readonly editSubmitButton: () => Effect.Effect<Locator>
    readonly deleteConfirmDialog: () => Effect.Effect<Locator>
    readonly deleteConfirmButton: () => Effect.Effect<Locator>
    readonly pagination: () => Effect.Effect<Locator>
    readonly paginationNext: () => Effect.Effect<Locator>
    readonly filter: (query: string) => Effect.Effect<void>
    readonly createOrganization: (name: string, slug?: string) => Effect.Effect<void>
    readonly updateOrganization: (name: string, newName: string) => Effect.Effect<void>
    readonly deleteOrganization: (name: string) => Effect.Effect<void>
    readonly goToNextPage: () => Effect.Effect<void>
  }
>() {}

export const OrganizationsPageLive = Layer.effect(
  OrganizationsPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-page')
        }),

      pageTitle: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-page-title')
        }),

      list: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-list')
        }),

      table: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-table')
        }),

      filterInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-filter-input')
        }),

      createButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-create-button')
        }),

      row: (name: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`organization-row-${name}`)
        }),

      editButton: (name: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`organization-edit-button-${name}`)
        }),

      deleteButton: (name: string) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`organization-delete-button-${name}`)
        }),

      createDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-create-dialog')
        }),

      createForm: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-create-form')
        }),

      createNameInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-create-name-input')
        }),

      createSlugInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-create-slug-input')
        }),

      createSubmitButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-create-submit-button')
        }),

      editDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-edit-dialog')
        }),

      editForm: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-edit-form')
        }),

      editNameInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-edit-name-input')
        }),

      editSubmitButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-edit-submit-button')
        }),

      deleteConfirmDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-delete-confirm-dialog')
        }),

      deleteConfirmButton: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organization-delete-confirm-button')
        }),

      pagination: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-pagination')
        }),

      paginationNext: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('organizations-pagination-next')
        }),

      filter: (query: string) =>
        Effect.gen(function* () {
          const filterInput = playwrightPage.getByTestId('organizations-filter-input')
          yield* LocatorHelpers.fill(filterInput, query)
        }),

      createOrganization: (name: string, slug?: string) =>
        Effect.gen(function* () {
          const createButton = playwrightPage.getByTestId('organizations-create-button')
          const createDialog = playwrightPage.getByTestId('organization-create-dialog')
          const nameInput = playwrightPage.getByTestId('organization-create-name-input')
          const submitButton = playwrightPage.getByTestId('organization-create-submit-button')
          
          yield* LocatorHelpers.click(createButton)
          yield* LocatorHelpers.waitForVisible(createDialog)
          yield* LocatorHelpers.fill(nameInput, name)
          if (slug) {
            const slugInput = playwrightPage.getByTestId('organization-create-slug-input')
            yield* LocatorHelpers.fill(slugInput, slug)
          }
          yield* LocatorHelpers.click(submitButton)
          yield* LocatorHelpers.waitForHidden(createDialog)
        }),

      updateOrganization: (name: string, newName: string) =>
        Effect.gen(function* () {
          const editButton = playwrightPage.getByTestId(`organization-edit-button-${name}`)
          const editDialog = playwrightPage.getByTestId('organization-edit-dialog')
          const nameInput = playwrightPage.getByTestId('organization-edit-name-input')
          const submitButton = playwrightPage.getByTestId('organization-edit-submit-button')
          
          yield* LocatorHelpers.click(editButton)
          yield* LocatorHelpers.waitForVisible(editDialog)
          yield* LocatorHelpers.fill(nameInput, newName)
          yield* LocatorHelpers.click(submitButton)
          yield* LocatorHelpers.waitForHidden(editDialog)
        }),

      deleteOrganization: (name: string) =>
        Effect.gen(function* () {
          const deleteButton = playwrightPage.getByTestId(`organization-delete-button-${name}`)
          const confirmDialog = playwrightPage.getByTestId('organization-delete-confirm-dialog')
          const confirmButton = playwrightPage.getByTestId('organization-delete-confirm-button')
          
          yield* LocatorHelpers.click(deleteButton)
          yield* LocatorHelpers.waitForVisible(confirmDialog)
          yield* LocatorHelpers.click(confirmButton)
          yield* LocatorHelpers.waitForHidden(confirmDialog)
        }),

      goToNextPage: () =>
        Effect.gen(function* () {
          const nextButton = playwrightPage.getByTestId('organizations-pagination-next')
          yield* LocatorHelpers.click(nextButton)
        }),
    }
  })
)

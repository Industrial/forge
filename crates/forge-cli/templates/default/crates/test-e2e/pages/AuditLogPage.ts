import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'

/**
 * AuditLogPage service - Effect.ts service for audit log page interactions
 */
export class AuditLogPage extends Context.Tag('AuditLogPage')<
  AuditLogPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly pageTitle: () => Effect.Effect<Locator>
    readonly list: () => Effect.Effect<Locator>
    readonly table: () => Effect.Effect<Locator>
    readonly filterInput: () => Effect.Effect<Locator>
    readonly pagination: () => Effect.Effect<Locator>
    readonly paginationNext: () => Effect.Effect<Locator>
    readonly paginationPrev: () => Effect.Effect<Locator>
    readonly paginationPage: (page: number) => Effect.Effect<Locator>
    readonly row: (index: number) => Effect.Effect<Locator>
    readonly viewButton: (index: number) => Effect.Effect<Locator>
    readonly detailsDialog: () => Effect.Effect<Locator>
    readonly filter: (query: string) => Effect.Effect<void>
    readonly goToNextPage: () => Effect.Effect<void>
    readonly goToPage: (page: number) => Effect.Effect<void>
    readonly viewEntryDetails: (index: number) => Effect.Effect<void>
  }
>() {}

export const AuditLogPageLive = Layer.effect(
  AuditLogPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-page')
        }),

      pageTitle: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-page-title')
        }),

      list: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-list')
        }),

      table: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-table')
        }),

      filterInput: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-filter-input')
        }),

      pagination: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-pagination')
        }),

      paginationNext: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-pagination-next')
        }),

      paginationPrev: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-pagination-prev')
        }),

      paginationPage: (page: number) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`audit-log-pagination-page-${page}`)
        }),

      row: (index: number) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`audit-log-row-${index}`)
        }),

      viewButton: (index: number) =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId(`audit-log-view-button-${index}`)
        }),

      detailsDialog: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('audit-log-details-dialog')
        }),

      filter: (query: string) =>
        Effect.gen(function* () {
          const filterInput = playwrightPage.getByTestId('audit-log-filter-input')
          yield* LocatorHelpers.fill(filterInput, query)
        }),

      goToNextPage: () =>
        Effect.gen(function* () {
          const nextButton = playwrightPage.getByTestId('audit-log-pagination-next')
          yield* LocatorHelpers.click(nextButton)
        }),

      goToPage: (page: number) =>
        Effect.gen(function* () {
          const pageButton = playwrightPage.getByTestId(`audit-log-pagination-page-${page}`)
          yield* LocatorHelpers.click(pageButton)
        }),

      viewEntryDetails: (index: number) =>
        Effect.gen(function* () {
          const row = playwrightPage.getByTestId(`audit-log-row-${index}`)
          const detailsDialog = playwrightPage.getByTestId('audit-log-details-dialog')
          
          yield* LocatorHelpers.click(row)
          yield* LocatorHelpers.waitForVisible(detailsDialog)
        }),
    }
  })
)

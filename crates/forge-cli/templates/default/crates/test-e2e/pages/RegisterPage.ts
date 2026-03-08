import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'

/**
 * RegisterPage service - Effect.ts service for registration page interactions
 * Methods take PlaywrightPage from context and return Effects
 */
export class RegisterPage extends Context.Tag('RegisterPage')<
  RegisterPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly form: () => Effect.Effect<Locator>
    readonly email: () => Effect.Effect<Locator>
    readonly password: () => Effect.Effect<Locator>
    readonly submit: () => Effect.Effect<Locator>
    readonly loginLink: () => Effect.Effect<Locator>
    readonly errorMessage: () => Effect.Effect<Locator>
    readonly register: (email: string, password: string) => Effect.Effect<void>
    readonly clickLoginLink: () => Effect.Effect<void>
  }
>() {}

/**
 * Create RegisterPage layer from PlaywrightPage
 */
export const RegisterPageLive = Layer.effect(
  RegisterPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('register-page')
        }),

      form: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('register-form')
        }),

      email: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('register-email-input')
        }),

      password: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('register-password-input')
        }),

      submit: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('register-submit-button')
        }),

      loginLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('register-login-link')
        }),

      errorMessage: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('register-error-message')
        }),

      register: (email: string, password: string) =>
        Effect.gen(function* () {
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('register-email-input').fill(email)
          )
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('register-password-input').fill(password)
          )
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('register-submit-button').click()
          )
          yield* Effect.promise(() =>
            playwrightPage.waitForURL('/authentication/login', { timeout: 5000 })
          )
        }),

      clickLoginLink: () =>
        Effect.gen(function* () {
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('register-login-link').click()
          )
        }),
    }
  })
)

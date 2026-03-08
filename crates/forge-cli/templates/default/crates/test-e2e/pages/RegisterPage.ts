import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

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
          const emailInput = playwrightPage.getByTestId('register-email-input')
          const passwordInput = playwrightPage.getByTestId(
            'register-password-input',
          )
          const submitButton = playwrightPage.getByTestId(
            'register-submit-button',
          )

          yield* LocatorHelpers.fill(emailInput, email)
          yield* LocatorHelpers.fill(passwordInput, password)
          yield* LocatorHelpers.click(submitButton)
          yield* PageHelpers.waitForURL(
            playwrightPage,
            '/authentication/login',
            { timeout: 5000 },
          )
        }),

      clickLoginLink: () =>
        Effect.gen(function* () {
          const loginLink = playwrightPage.getByTestId('register-login-link')
          yield* LocatorHelpers.click(loginLink)
        }),
    }
  }),
)

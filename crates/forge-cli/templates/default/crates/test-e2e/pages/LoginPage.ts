import { Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

import { PlaywrightPage } from '@/fixtures/playwright'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

/**
 * LoginPage service - Effect.ts service for login page interactions
 * Methods take PlaywrightPage from context and return Effects
 */
export class LoginPage extends Context.Tag('LoginPage')<
  LoginPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly form: () => Effect.Effect<Locator>
    readonly email: () => Effect.Effect<Locator>
    readonly password: () => Effect.Effect<Locator>
    readonly submit: () => Effect.Effect<Locator>
    readonly registerLink: () => Effect.Effect<Locator>
    readonly errorMessage: () => Effect.Effect<Locator>
    readonly login: (email: string, password: string) => Effect.Effect<void>
    readonly clickRegisterLink: () => Effect.Effect<void>
  }
>() {}

/**
 * Create LoginPage layer from PlaywrightPage
 */
export const LoginPageLive = Layer.effect(
  LoginPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-page')
        }),

      form: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-form')
        }),

      email: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-email-input')
        }),

      password: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-password-input')
        }),

      submit: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-submit-button')
        }),

      registerLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-register-link')
        }),

      errorMessage: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-error-message')
        }),

      login: (email: string, password: string) =>
        Effect.gen(function* () {
          const emailInput = playwrightPage.getByTestId('login-email-input')
          const passwordInput = playwrightPage.getByTestId('login-password-input')
          const submitButton = playwrightPage.getByTestId('login-submit-button')
          
          yield* LocatorHelpers.fill(emailInput, email)
          yield* LocatorHelpers.fill(passwordInput, password)
          yield* LocatorHelpers.click(submitButton)
          yield* PageHelpers.waitForURL(playwrightPage, /\/dashboard|\/authentication\/select-scope/, {
            timeout: 5000,
          })
        }),

      clickRegisterLink: () =>
        Effect.gen(function* () {
          const registerLink = playwrightPage.getByTestId('login-register-link')
          yield* LocatorHelpers.click(registerLink)
        }),
    }
  })
)

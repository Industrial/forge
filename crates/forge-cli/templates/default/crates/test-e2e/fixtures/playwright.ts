import { Page } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

/**
 * Playwright Page service - provides Page instance to Effect programs
 * This enables Effect.ts composition for E2E tests
 */
export class PlaywrightPage extends Context.Tag('PlaywrightPage')<
  PlaywrightPage,
  Page
>() {}

/**
 * Create a Layer that provides PlaywrightPage
 * Use this to inject Page into Effect programs
 */
export const createPlaywrightPageLayer = (page: Page): Layer.Layer<PlaywrightPage> =>
  Layer.succeed(PlaywrightPage, page)

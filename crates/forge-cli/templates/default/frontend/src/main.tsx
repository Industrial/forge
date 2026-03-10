import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { Effect, Layer, pipe } from 'effect'
import { EffectRuntimeProvider } from 'react-effect-hooks'

import '@/reset.css'
import App from '@/App.tsx'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'

await Effect.runPromise(
  pipe(
    pipe(
      Effect.gen(function* () {
        yield* Effect.logInfo('Starting application')

        const auth = yield* Authentication
        yield* auth.restoreSession()

        const rootElement = document.getElementById('root')
        if (rootElement == null) {
          return yield* Effect.fail(
            new Error('Failed to find the root element'),
          )
        }

        yield* Effect.sync(() => {
          const layer = getApplicationLayer()
          const runtime = Effect.runSync(Effect.scoped(Layer.toRuntime(layer)))
          ReactDOM.createRoot(rootElement).render(
            <BrowserRouter>
              <EffectRuntimeProvider runtime={runtime}>
                <App />
              </EffectRuntimeProvider>
            </BrowserRouter>,
          )
        })
      }),
      Effect.provide(getApplicationLayer()),
    ),
    Effect.tapError((error) =>
      Effect.sync(() => {
        console.error('Application failed to start:', error)
      }),
    ),
  ),
)

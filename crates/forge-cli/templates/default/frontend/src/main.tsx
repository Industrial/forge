import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { Effect, SubscriptionRef } from 'effect'

import '@/reset.css'
import App from '@/App.tsx'
import { buildApplicationLayer, setApplicationLayer } from '@/lib/appLayer'
import { createReactiveStoreFromRef } from '@/lib/ReactiveStore'
import {
  AuthenticationStateReactiveStoreTag,
  initialAuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { Authentication } from '@/features/authentication/services/Authentication'

// Run app in a long-lived scope so the auth store ref is created once and shared
// by restoreSession and by React. Otherwise each Effect.provide(layer) would
// create a new scope and a new ref, so refresh would show stale initial state.
await Effect.runPromise(
  Effect.scoped(
    Effect.gen(function* () {
      const authRef = yield* SubscriptionRef.make(initialAuthenticationState)
      const authStore = createReactiveStoreFromRef(
        authRef,
        AuthenticationStateReactiveStoreTag,
      )
      const layer = buildApplicationLayer(authStore)
      setApplicationLayer(layer)

      yield* Effect.gen(function* () {
        yield* Effect.logInfo('Starting application')
        const auth = yield* Authentication
        yield* auth.restoreSession()

        const rootElement = document.getElementById('root')
        if (!rootElement) {
          throw new Error('Failed to find the root element')
        }

        ReactDOM.createRoot(rootElement).render(
          <BrowserRouter>
            <App />
          </BrowserRouter>,
        )

        yield* Effect.never
      }).pipe(Effect.provide(layer))
    }),
  ),
)

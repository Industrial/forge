import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { Effect } from 'effect'

import '@/reset.css'
import App from '@/App.tsx'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'

const layer = getApplicationLayer()

await Effect.runPromise(
  Effect.gen(function* () {
    const auth = yield* Authentication
    yield* auth.restoreSession()
  }).pipe(Effect.provide(layer)),
)

const rootElement = document.getElementById('root')
if (!rootElement) {
  throw new Error('Failed to find the root element')
}

ReactDOM.createRoot(rootElement).render(
  <BrowserRouter>
    <App />
  </BrowserRouter>,
)

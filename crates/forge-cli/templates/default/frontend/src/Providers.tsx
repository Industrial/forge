import CssBaseline from '@mui/material/CssBaseline'
import type { Theme } from '@emotion/react'
import { ThemeProvider } from '@mui/material/styles'
import { useEffect, type ReactNode } from 'react'

import { Effect } from 'effect'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'

export type ProvidersProps = {
  theme: Theme
  children: ReactNode
}

function ProvidersInner({ theme, children }: ProvidersProps) {
  useEffect(() => {
    const layer = getApplicationLayer()
    Effect.runPromise(
      Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.getCurrentUser()
      }).pipe(Effect.provide(layer)),
    ).catch(() => {})
  }, [])

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      {children}
    </ThemeProvider>
  )
}

export function Providers({ theme, children }: ProvidersProps) {
  return <ProvidersInner theme={theme}>{children}</ProvidersInner>
}

import CssBaseline from '@mui/material/CssBaseline'
import type { Theme } from '@emotion/react'
import { ThemeProvider } from '@mui/material/styles'
import type { ReactNode } from 'react'
import { useComponentLogger } from '@/hooks'

export type ProvidersProps = {
  theme: Theme
  children: ReactNode
}

export function Providers({ theme, children }: ProvidersProps) {
  useComponentLogger('Providers')
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      {children}
    </ThemeProvider>
  )
}

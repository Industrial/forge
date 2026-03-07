import CssBaseline from '@mui/material/CssBaseline'
import type { Theme } from '@emotion/react'
import { ThemeProvider } from '@mui/material/styles'
import type { ReactNode } from 'react'

export type ProvidersProps = {
  theme: Theme
  children: ReactNode
}

export function Providers({ theme, children }: ProvidersProps) {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      {children}
    </ThemeProvider>
  )
}

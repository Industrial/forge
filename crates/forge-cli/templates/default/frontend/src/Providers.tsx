import type { Theme } from "@emotion/react"
import { ThemeProvider } from "@mui/material/styles"
import CssBaseline from "@mui/material/CssBaseline"

// import { AuthenticationRuntimeProvider } from "@/lib/AuthenticationRuntimeProvider"
import { ForgeWebsocketProvider } from "@/context/ForgeWebsocketContext"
import { AuthenticationProvider } from "@/context/AuthenticationContext"

export type ProvidersProps = {
  theme: Theme
  children: React.ReactNode
}

export function Providers({ theme, children }: ProvidersProps) {
  console.log('Providers', theme, children)

  return (
    // <AuthenticationRuntimeProvider>
      <AuthenticationProvider>
        <ForgeWebsocketProvider>
          <ThemeProvider theme={theme}>
            <CssBaseline />
            {children}
          </ThemeProvider>
        </ForgeWebsocketProvider>
      </AuthenticationProvider>
    // </AuthenticationRuntimeProvider>
  )
}

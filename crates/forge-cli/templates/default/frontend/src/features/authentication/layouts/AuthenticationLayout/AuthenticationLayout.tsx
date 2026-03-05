import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Typography from '@mui/material/Typography'
import React from 'react'
import { useSession } from '../../../../context/Session'

type AuthLayoutProps = { children: React.ReactNode }

export default function AuthenticationLayout({ children }: AuthLayoutProps) {
  const { flash } = useSession()

  return (
    <Box
      sx={{
        minHeight: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        p: 3,
      }}
    >
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          gap: 3,
          maxWidth: 400,
          width: '100%',
        }}
      >
        {(flash?.message ?? flash?.error) != null && (
          <>
            {flash?.message != null && (
              <Alert severity="success">{flash.message}</Alert>
            )}
            {flash?.error != null && (
              <Alert severity="error">
                <Typography variant="subtitle2">Error</Typography>
                {flash.error}
              </Alert>
            )}
          </>
        )}
        {children}
      </Box>
    </Box>
  )
}

import Box from '@mui/material/Box'
import React from 'react'

export type AuthenticationLayoutProps = {
  children: React.ReactNode
}

export default function AuthenticationLayout({
  children,
}: AuthenticationLayoutProps) {
  console.log('AuthenticationLayout')

  // const { flash } = useAuthentication()

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
        {/* {(flash?.message ?? flash?.error) != null && (
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
        )} */}
        {children}
      </Box>
    </Box>
  )
}

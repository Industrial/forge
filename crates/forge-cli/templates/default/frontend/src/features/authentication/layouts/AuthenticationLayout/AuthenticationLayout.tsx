import Box from '@mui/material/Box'
import React from 'react'

export type AuthenticationLayoutProps = {
  children: React.ReactNode
}

export default function AuthenticationLayout({
  children,
}: AuthenticationLayoutProps) {
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
        {children}
      </Box>
    </Box>
  )
}

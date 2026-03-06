import React from 'react'
import Box from '@mui/material/Box'
import Navbar from '../components/Navbar'
import { SubscriptionStreamRunner } from '../components/SubscriptionStreamRunner'

export type LayoutProps = {
  children: React.ReactNode
  colorScheme: 'light' | 'dark'
  onToggleTheme: () => void
}

export default function Layout({
  children,
  colorScheme,
  onToggleTheme,
}: LayoutProps) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        minHeight: '100%',
        height: '100%',
        overflow: 'auto',
        flexGrow: 1,
      }}
    >
      <SubscriptionStreamRunner />
      <Navbar
        appName="App"
        colorScheme={colorScheme}
        onToggleTheme={onToggleTheme}
      />
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          gap: 2,
          m: 2,
          p: 2,
          bgcolor: 'background.paper',
          borderRadius: 1,
          flexGrow: 1,
        }}
      >
        {children}
      </Box>
    </Box>
  )
}

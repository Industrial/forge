import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import Box from '@mui/material/Box'
import { useAuthentication } from '../context/AuthenticationContext'
import LoadingSpinner from './LoadingSpinner'

type GuestRouteProps = { children: React.ReactNode }

/**
 * Renders children only when the user is not logged in.
 * Redirects to /dashboard (or saved "from" location) when the user is authenticated.
 * Use for login and register pages.
 */
export default function GuestRoute({ children }: GuestRouteProps) {
  const { user, loading, needs_scope_select } = useAuthentication()
  const location = useLocation()

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', flex: 1, minHeight: '40vh' }}>
        <LoadingSpinner />
      </Box>
    )
  }

  if (user != null) {
    const to =
      (location.state as { from?: { pathname: string } } | null)?.from
        ?.pathname ?? '/dashboard'
    if (needs_scope_select) {
      return (
        <Navigate
          to="/authentication/select-scope"
          state={{ from: { pathname: to } }}
          replace
        />
      )
    }
    return <Navigate to={to} replace />
  }

  return <>{children}</>
}

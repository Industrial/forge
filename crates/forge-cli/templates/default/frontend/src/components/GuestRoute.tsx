import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuthentication } from '../context/AuthenticationContext'

type GuestRouteProps = { children: React.ReactNode }

/**
 * Renders children only when the user is not logged in.
 * Redirects to /dashboard (or saved "from" location) when the user is authenticated.
 * Use for login and register pages.
 */
export default function GuestRoute({ children }: GuestRouteProps) {
  const { user, loading } = useAuthentication()
  const location = useLocation()

  if (loading) {
    return null
  }

  if (user != null) {
    const to =
      (location.state as { from?: { pathname: string } } | null)?.from
        ?.pathname ?? '/dashboard'
    return <Navigate to={to} replace />
  }

  return <>{children}</>
}

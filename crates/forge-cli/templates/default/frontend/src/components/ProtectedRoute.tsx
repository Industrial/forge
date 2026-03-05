import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuthentication } from '../context/AuthenticationContext'

type ProtectedRouteProps = { children: React.ReactNode }

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { user, loading } = useAuthentication()
  const location = useLocation()

  if (loading) {
    return null // or a small loading spinner
  }

  if (user == null) {
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  return <>{children}</>
}

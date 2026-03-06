import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import Box from '@mui/material/Box'
import { useAuthentication } from '../context/AuthenticationContext'
import LoadingSpinner from './LoadingSpinner'

type ProtectedRouteProps = { children: React.ReactNode }

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  console.log('ProtectedRoute')

  const { user, loading } = useAuthentication()
  const location = useLocation()

  if (loading) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          flex: 1,
          minHeight: '40vh',
        }}
      >
        <LoadingSpinner />
      </Box>
    )
  }

  if (user == null) {
    return (
      <Navigate to="/authentication/login" state={{ from: location }} replace />
    )
  }

  return <>{children}</>
}

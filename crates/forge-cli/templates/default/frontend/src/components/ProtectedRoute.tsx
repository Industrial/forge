import React from 'react'
import { Navigate } from 'react-router-dom'
import { Option } from 'effect'

import { useAuthStoreWithInit } from '@/features/authentication/stores'

export type ProtectedRouteProps = {
  children: React.ReactNode
}

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { authentication, initialized } = useAuthStoreWithInit()
  const isUserAuthenticated = Option.isSome(authentication.user)

  if (!initialized) {
    return null
  }

  if (!isUserAuthenticated) {
    return <Navigate to="/authentication/login" replace />
  }

  return <>{children}</>
}

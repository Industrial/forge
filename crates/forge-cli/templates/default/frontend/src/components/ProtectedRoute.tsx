import React from 'react'
import { Navigate } from 'react-router-dom'
import { Option } from 'effect'

import { useAuthenticationStateReactiveStore } from '@/features/authentication/stores'

export type ProtectedRouteProps = {
  children: React.ReactNode
}

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  const authentication = useAuthenticationStateReactiveStore()
  const isUserAuthenticated = Option.isSome(authentication.user)

  if (!isUserAuthenticated) {
    console.log('ProtectedRoute: redirecting to login')

    return <Navigate to="/authentication/login" replace />
  }

  return <>{children}</>
}

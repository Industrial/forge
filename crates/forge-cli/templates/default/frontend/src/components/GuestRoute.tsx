import React from 'react'
import { Navigate } from 'react-router-dom'
import { Option } from 'effect'

import { useAuthStore } from '@/features/authentication/stores'

export type GuestRouteProps = {
  children: React.ReactNode
}

export default function GuestRoute({ children }: GuestRouteProps) {
  const authentication = useAuthStore()
  const isUserAuthenticated = Option.isSome(authentication.user)

  if (isUserAuthenticated) {
    return <Navigate to="/" replace />
  }

  return <>{children}</>
}

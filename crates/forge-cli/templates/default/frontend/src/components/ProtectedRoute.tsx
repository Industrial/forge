import React from 'react'
import { useComponentLogger } from '@/hooks'
import { RouteGuard } from './RouteGuard'

export type ProtectedRouteProps = {
  children: React.ReactNode
}

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  useComponentLogger('ProtectedRoute')
  return (
    <RouteGuard
      requireAuth
      requireAuthWithInit
      redirectTo="/authentication/login"
    >
      {children}
    </RouteGuard>
  )
}

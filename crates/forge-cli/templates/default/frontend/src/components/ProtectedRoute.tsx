import type React from 'react'
import { RouteGuard } from './RouteGuard'

export type ProtectedRouteProps = {
  children: React.ReactNode
}

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
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

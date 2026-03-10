import type React from 'react'
import { RouteGuard } from './RouteGuard'

export type GuestRouteProps = {
  children: React.ReactNode
}

export default function GuestRoute({ children }: GuestRouteProps) {
  return (
    <RouteGuard redirectIfAuthenticated="/dashboard">{children}</RouteGuard>
  )
}

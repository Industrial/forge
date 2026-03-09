import React from 'react'
import { useComponentLogger } from '@/hooks'
import { RouteGuard } from './RouteGuard'

export type GuestRouteProps = {
  children: React.ReactNode
}

export default function GuestRoute({ children }: GuestRouteProps) {
  useComponentLogger('GuestRoute')
  return (
    <RouteGuard redirectIfAuthenticated="/dashboard">{children}</RouteGuard>
  )
}

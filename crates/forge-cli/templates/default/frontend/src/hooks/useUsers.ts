/**
 * useUsers – users list from fetchUsersEffect via the Effect runtime.
 *
 * Run fetchUsersEffect on mount and when refresh() is called.
 * Use inside a tree that has EffectRuntimeProvider with HttpClient (e.g. AuthenticationRuntimeProvider).
 */
import { useCallback, useEffect, useState } from 'react'
import { useEffectRuntime } from '../lib/react-effect'
import { runWithAppRuntime, type AppServices } from '../lib/appLayer'
import { fetchUsersEffect, type User } from '../effects/users'

export interface UseUsersResult {
  users: User[]
  loading: boolean
  error: string | null
  refresh: () => void
}

export function useUsers(canRead: boolean): UseUsersResult {
  const { runtime } = useEffectRuntime<AppServices>()
  const [users, setUsers] = useState<User[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(() => {
    if (!canRead) {
      setUsers([])
      setLoading(false)
      setError(null)
      return
    }
    setLoading(true)
    setError(null)
    runWithAppRuntime(runtime, fetchUsersEffect)
      .then((list) => {
        setUsers(list)
        setLoading(false)
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : 'Failed to load users.')
        setUsers([])
        setLoading(false)
      })
  }, [runtime, canRead])

  useEffect(() => {
    refresh()
  }, [refresh])

  return { users, loading, error, refresh }
}

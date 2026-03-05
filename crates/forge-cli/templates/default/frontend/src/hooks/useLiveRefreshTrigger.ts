import { useState } from 'react'
import { useLiveUpdates } from '../context/LiveWs'
import type { LiveUpdateKey } from '../context/LiveWs'

/**
 * Subscribes to live updates for a channel and exposes a trigger value that
 * increments on each update. Use `trigger` in a dependency array to re-run
 * effects (e.g. refetch) when live data arrives.
 */
export function useLiveRefreshTrigger(
  channel: LiveUpdateKey,
): { trigger: number; connected: boolean } {
  const [trigger, setTrigger] = useState(0)
  const { connected } = useLiveUpdates(channel, () => {
    setTrigger((n) => n + 1)
  })
  return { trigger, connected }
}

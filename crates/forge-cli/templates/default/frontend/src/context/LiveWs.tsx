import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react'
import { Effect } from 'effect'
import { useEffectRuntime } from '../lib/react-effect'
import { runWithAppRuntime, type AppServices } from '../lib/appLayer'
import { ForgeWebsocket } from '../services/ForgeWebsocket'

/** Keys used to dispatch live updates (maps message type/resource to listener key). */
export type LiveUpdateKey =
  | 'audit-log'
  | 'users'
  | 'roles'
  | 'role_permissions'
  | 'organizations'

type Listener = (data: unknown) => void

type LiveWsContextValue = {
  connected: boolean
  /** Subscribe to live updates for a key. Listener receives the parsed message. Returns unsubscribe. */
  subscribe: (key: LiveUpdateKey, listener: Listener) => () => void
}

const LiveWsContext = createContext<LiveWsContextValue | null>(null)

export function useLiveWs(): LiveWsContextValue {
  const ctx = useContext(LiveWsContext)
  if (!ctx) throw new Error('useLiveWs must be used within LiveWsProvider')
  return ctx
}

/**
 * Subscribe to live updates for a given key. Callback receives the parsed message
 * (e.g. LiveEvent or resource_changed). Use to refetch or update state.
 */
export function useLiveUpdates(
  key: LiveUpdateKey,
  onUpdate: (data: unknown) => void,
): { connected: boolean } {
  const { connected, subscribe } = useLiveWs()
  const onUpdateRef = useRef(onUpdate)
  onUpdateRef.current = onUpdate
  useEffect(() => {
    const stable = (data: unknown) => onUpdateRef.current?.(data)
    return subscribe(key, stable)
  }, [key, subscribe])
  return { connected }
}

const POLL_INTERVAL_MS = 2000

/**
 * LiveWs provider backed by the Effect runtime's ForgeWebsocket service.
 * Must be used inside AuthenticationRuntimeProvider so the runtime includes Websocket + ForgeWebsocket.
 */
export function LiveWsProvider({ children }: { children: React.ReactNode }) {
  const { runtime } = useEffectRuntime<AppServices>()
  const [connected, setConnected] = useState(false)

  const getConnectedEffect = Effect.gen(function* () {
    const fw = yield* ForgeWebsocket
    return yield* fw.getConnected()
  })

  useEffect(() => {
    if (runtime == null) return
    const updateConnected = () => {
      runWithAppRuntime(runtime, getConnectedEffect)
        .then(setConnected)
        .catch(() => setConnected(false))
    }
    updateConnected()
    const id = setInterval(updateConnected, POLL_INTERVAL_MS)
    return () => clearInterval(id)
  }, [runtime])

  const subscribe = useCallback(
    (key: LiveUpdateKey, listener: Listener) => {
      if (runtime == null) return () => {}
      const state: { unsub: (() => void) | null; cancelled: boolean } = {
        unsub: null,
        cancelled: false,
      }
      runWithAppRuntime(
        runtime,
        Effect.gen(function* () {
          const fw = yield* ForgeWebsocket
          return yield* fw.subscribe(key, listener)
        }),
      ).then((fn) => {
        state.unsub = fn
        if (state.cancelled) fn()
      })
      return () => {
        state.cancelled = true
        if (state.unsub) {
          state.unsub()
          state.unsub = null
        }
      }
    },
    [runtime],
  )

  const value: LiveWsContextValue = React.useMemo(
    () => ({ connected, subscribe }),
    [connected, subscribe],
  )

  return (
    <LiveWsContext.Provider value={value}>{children}</LiveWsContext.Provider>
  )
}

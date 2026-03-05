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
import {
  ForgeWebsocket,
  type ForgeWebsocketKey,
} from '../services/ForgeWebsocket'

export type { ForgeWebsocketKey }

type Listener = (data: unknown) => void

type ForgeWebsocketContextValue = {
  connected: boolean
  /** Subscribe to live updates for a key. Listener receives the parsed message. Returns unsubscribe. */
  subscribe: (key: ForgeWebsocketKey, listener: Listener) => () => void
}

const ForgeWebsocketContext = createContext<ForgeWebsocketContextValue | null>(
  null,
)

export function useForgeWebsocketContext(): ForgeWebsocketContextValue {
  const ctx = useContext(ForgeWebsocketContext)
  if (!ctx) {
    throw new Error(
      'useForgeWebsocketContext must be used within ForgeWebsocketProvider',
    )
  }
  return ctx
}

/**
 * Subscribe to live updates for a given key. Callback receives the parsed message
 * (e.g. LiveEvent or resource_changed). Use to refetch or update state.
 */
export function useLiveUpdates(
  key: ForgeWebsocketKey,
  onUpdate: (data: unknown) => void,
): { connected: boolean } {
  const { connected, subscribe } = useForgeWebsocketContext()
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
 * ForgeWebsocket provider backed by the Effect runtime's ForgeWebsocket service.
 * Must be used inside AuthenticationRuntimeProvider so the runtime includes Websocket + ForgeWebsocket.
 */
export function ForgeWebsocketProvider({
  children,
}: {
  children: React.ReactNode
}) {
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
    (key: ForgeWebsocketKey, listener: Listener) => {
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

  const value: ForgeWebsocketContextValue = React.useMemo(
    () => ({ connected, subscribe }),
    [connected, subscribe],
  )

  return (
    <ForgeWebsocketContext.Provider value={value}>
      {children}
    </ForgeWebsocketContext.Provider>
  )
}

/**
 * Reactive store for subscription stream connection status.
 * SubscriptionStreamRunner sets connected=true on "ready", false when stream ends.
 * useLiveRefreshTrigger reads connected for the live indicator.
 */
import { makeReactiveStore, type ReactiveStore } from '@/lib/ReactiveStore'

export interface SubscriptionStreamStatus {
  connected: boolean
}

export const initialSubscriptionStreamStatus: SubscriptionStreamStatus = {
  connected: false,
}

const {
  tag: SubscriptionStreamStatusStoreTag,
  layer: subscriptionStreamStatusStoreLayer,
} = makeReactiveStore(
  '@forge/SubscriptionStreamStatusStore',
  initialSubscriptionStreamStatus,
)

export { SubscriptionStreamStatusStoreTag }

export type SubscriptionStreamStatusStore =
  ReactiveStore<SubscriptionStreamStatus>

export function getSubscriptionStreamStatusStoreLayer() {
  return subscriptionStreamStatusStoreLayer
}

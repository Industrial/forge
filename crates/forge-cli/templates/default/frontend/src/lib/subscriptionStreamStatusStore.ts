/**
 * Reactive store for subscription stream connection status.
 * SubscriptionStreamRunner sets connected=true on "ready", false when stream ends.
 * useLiveRefreshTrigger reads connected for the live indicator.
 * DSL: same pattern as auth store — merge layer in app layer, use tag in Effect.
 */
import { defineStore, type ReactiveStore } from '@/lib/ReactiveStore'

export interface SubscriptionStreamStatus {
  connected: boolean
}

export const initialSubscriptionStreamStatus: SubscriptionStreamStatus = {
  connected: false,
}

/** DSL: one store definition; use .tag in Effect, .layer in app layer. */
export const SubscriptionStreamStatusStore = defineStore(
  '@forge/SubscriptionStreamStatusStore',
  initialSubscriptionStreamStatus,
)

export const SubscriptionStreamStatusStoreTag =
  SubscriptionStreamStatusStore.tag
export const subscriptionStreamStatusStoreLayer =
  SubscriptionStreamStatusStore.layer

export type SubscriptionStreamStatusStore =
  ReactiveStore<SubscriptionStreamStatus>

export function getSubscriptionStreamStatusStoreLayer() {
  return subscriptionStreamStatusStoreLayer
}

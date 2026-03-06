/**
 * In-memory registry for subscription_id → refetch callback.
 * When the subscription stream emits an invalidation for a subscription_id,
 * we look up and call the registered onInvalidate so the view refetches with stored params.
 *
 * @see SubscriptionStream, useEntitySubscription (Epic 8)
 */

import type { ListQueryParams } from '../services/EntityApi'

export type SubscriptionEntry = {
  entityId: string
  params: ListQueryParams | undefined
  onInvalidate: () => void
}

const map = new Map<string, SubscriptionEntry>()

export function register(
  subscriptionId: string,
  entry: SubscriptionEntry,
): void {
  map.set(subscriptionId, entry)
}

export function unregister(subscriptionId: string): void {
  map.delete(subscriptionId)
}

export function trigger(subscriptionId: string): void {
  const entry = map.get(subscriptionId)
  if (entry) entry.onInvalidate()
}

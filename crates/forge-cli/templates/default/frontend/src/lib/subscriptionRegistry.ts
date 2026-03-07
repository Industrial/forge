/**
 * In-memory registry for SSE subscription invalidations.
 * When the subscription stream receives a subscription_id, trigger(id) runs
 * all registered onInvalidate callbacks for that id.
 *
 * @see SubscriptionStream, useEntitySubscription, useLiveRefreshTrigger (Epic 8)
 */

export interface SubscriptionEntry {
  entityId: string
  params?: unknown
  onInvalidate: () => void
}

const registry = new Map<string, SubscriptionEntry>()

export function register(
  subscriptionId: string,
  entry: SubscriptionEntry,
): void {
  registry.set(subscriptionId, entry)
}

export function unregister(subscriptionId: string): void {
  registry.delete(subscriptionId)
}

export function trigger(subscriptionId: string): void {
  const entry = registry.get(subscriptionId)
  if (entry) {
    entry.onInvalidate()
  }
}

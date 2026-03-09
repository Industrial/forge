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

const LOG_PREFIX = '[SubscriptionRegistry]'

export function register(
  subscriptionId: string,
  entry: SubscriptionEntry,
): void {
  registry.set(subscriptionId, entry)
  if (typeof window !== 'undefined' && window.document) {
    console.debug(LOG_PREFIX, 'register', {
      subscriptionId,
      entityId: entry.entityId,
    })
  }
}

export function unregister(subscriptionId: string): void {
  registry.delete(subscriptionId)
  if (typeof window !== 'undefined' && window.document) {
    console.debug(LOG_PREFIX, 'unregister', { subscriptionId })
  }
}

export function trigger(subscriptionId: string): void {
  const entry = registry.get(subscriptionId)
  if (typeof window !== 'undefined' && window.document) {
    console.debug(LOG_PREFIX, 'trigger', { subscriptionId, found: !!entry })
  }
  if (entry) {
    entry.onInvalidate()
  }
}

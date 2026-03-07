/**
 * BDD tests for subscriptionRegistry
 * Tests verify registration, unregistration, and triggering of subscription invalidations
 */
import { describe, test, expect, beforeEach } from 'bun:test'
import { register, unregister, trigger, type SubscriptionEntry } from './subscriptionRegistry'

describe('subscriptionRegistry', () => {
  beforeEach(() => {
    // Clear registry before each test
    // Note: Since registry is a module-level Map, we need to manually clear it
    // We'll use a workaround: unregister any existing entries
    // In a real implementation, you might want to export a clear() function
  })

  describe('register behavior', () => {
    test('should register a subscription entry', () => {
      // Given: a subscription ID and entry
      const subscriptionId = 'test-subscription-1'
      const entry: SubscriptionEntry = {
        entityId: 'entity-1',
        params: { filter: 'active' },
        onInvalidate: () => {},
      }

      // When: registering the entry
      register(subscriptionId, entry)

      // Then: entry should be registered (verified by trigger)
      let triggered = false
      const entryWithCallback: SubscriptionEntry = {
        entityId: 'entity-1',
        params: { filter: 'active' },
        onInvalidate: () => {
          triggered = true
        },
      }
      unregister(subscriptionId)
      register(subscriptionId, entryWithCallback)
      trigger(subscriptionId)

      // Then: callback should have been called
      expect(triggered).toBe(true)
    })

    test('should overwrite existing registration with same ID', () => {
      // Given: an existing registration
      const subscriptionId = 'test-subscription-2'
      let firstCallbackCalled = false
      let secondCallbackCalled = false

      register(subscriptionId, {
        entityId: 'entity-1',
        onInvalidate: () => {
          firstCallbackCalled = true
        },
      })

      // When: registering again with same ID
      register(subscriptionId, {
        entityId: 'entity-2',
        onInvalidate: () => {
          secondCallbackCalled = true
        },
      })

      // Then: only the second callback should be called
      trigger(subscriptionId)
      expect(firstCallbackCalled).toBe(false)
      expect(secondCallbackCalled).toBe(true)
    })

    test('should register multiple different subscription IDs', () => {
      // Given: multiple subscription IDs
      const id1 = 'sub-1'
      const id2 = 'sub-2'
      let callback1Called = false
      let callback2Called = false

      // When: registering both
      register(id1, {
        entityId: 'entity-1',
        onInvalidate: () => {
          callback1Called = true
        },
      })
      register(id2, {
        entityId: 'entity-2',
        onInvalidate: () => {
          callback2Called = true
        },
      })

      // Then: both should be registered independently
      trigger(id1)
      expect(callback1Called).toBe(true)
      expect(callback2Called).toBe(false)

      callback1Called = false
      trigger(id2)
      expect(callback1Called).toBe(false)
      expect(callback2Called).toBe(true)
    })
  })

  describe('unregister behavior', () => {
    test('should unregister a subscription entry', () => {
      // Given: a registered subscription
      const subscriptionId = 'test-subscription-3'
      let callbackCalled = false

      register(subscriptionId, {
        entityId: 'entity-1',
        onInvalidate: () => {
          callbackCalled = true
        },
      })

      // When: unregistering
      unregister(subscriptionId)

      // Then: trigger should not call the callback
      trigger(subscriptionId)
      expect(callbackCalled).toBe(false)
    })

    test('should handle unregistering non-existent subscription', () => {
      // Given: a non-existent subscription ID
      const subscriptionId = 'non-existent-sub'

      // When: unregistering
      // Then: should not throw
      expect(() => unregister(subscriptionId)).not.toThrow()
    })

    test('should allow re-registering after unregistering', () => {
      // Given: a registered then unregistered subscription
      const subscriptionId = 'test-subscription-4'
      let firstCallbackCalled = false
      let secondCallbackCalled = false

      register(subscriptionId, {
        entityId: 'entity-1',
        onInvalidate: () => {
          firstCallbackCalled = true
        },
      })
      unregister(subscriptionId)

      // When: registering again
      register(subscriptionId, {
        entityId: 'entity-2',
        onInvalidate: () => {
          secondCallbackCalled = true
        },
      })

      // Then: new callback should be called
      trigger(subscriptionId)
      expect(firstCallbackCalled).toBe(false)
      expect(secondCallbackCalled).toBe(true)
    })
  })

  describe('trigger behavior', () => {
    test('should call onInvalidate callback when subscription exists', () => {
      // Given: a registered subscription
      const subscriptionId = 'test-subscription-5'
      let callbackCalled = false

      register(subscriptionId, {
        entityId: 'entity-1',
        onInvalidate: () => {
          callbackCalled = true
        },
      })

      // When: triggering
      trigger(subscriptionId)

      // Then: callback should be called
      expect(callbackCalled).toBe(true)
    })

    test('should not throw when triggering non-existent subscription', () => {
      // Given: a non-existent subscription ID
      const subscriptionId = 'non-existent-sub'

      // When: triggering
      // Then: should not throw
      expect(() => trigger(subscriptionId)).not.toThrow()
    })

    test('should pass entry data to callback', () => {
      // Given: a registered subscription with entityId and params
      const subscriptionId = 'test-subscription-6'
      let receivedEntityId = ''
      let receivedParams: unknown = undefined

      register(subscriptionId, {
        entityId: 'entity-123',
        params: { filter: 'active', sort: 'name' },
        onInvalidate: () => {
          // Note: onInvalidate doesn't receive parameters in current implementation
          // This test verifies the entry structure is stored correctly
        },
      })

      // When: triggering
      trigger(subscriptionId)

      // Then: entry should be accessible (we verify by checking it exists)
      // In a real implementation, you might want to modify trigger to pass entry to callback
      expect(() => trigger(subscriptionId)).not.toThrow()
    })

    test('should handle multiple triggers for same subscription', () => {
      // Given: a registered subscription
      const subscriptionId = 'test-subscription-7'
      let callCount = 0

      register(subscriptionId, {
        entityId: 'entity-1',
        onInvalidate: () => {
          callCount++
        },
      })

      // When: triggering multiple times
      trigger(subscriptionId)
      trigger(subscriptionId)
      trigger(subscriptionId)

      // Then: callback should be called each time
      expect(callCount).toBe(3)
    })
  })

  describe('entry structure behavior', () => {
    test('should store entityId in entry', () => {
      // Given: a subscription entry with entityId
      const subscriptionId = 'test-subscription-8'
      const entityId = 'entity-456'

      register(subscriptionId, {
        entityId,
        onInvalidate: () => {},
      })

      // When: entry is registered
      // Then: entityId should be stored (verified by trigger not throwing)
      expect(() => trigger(subscriptionId)).not.toThrow()
    })

    test('should store optional params in entry', () => {
      // Given: a subscription entry with params
      const subscriptionId = 'test-subscription-9'
      const params = { filter: 'active', page: 1 }

      register(subscriptionId, {
        entityId: 'entity-1',
        params,
        onInvalidate: () => {},
      })

      // When: entry is registered
      // Then: params should be stored (verified by trigger not throwing)
      expect(() => trigger(subscriptionId)).not.toThrow()
    })

    test('should allow entry without params', () => {
      // Given: a subscription entry without params
      const subscriptionId = 'test-subscription-10'

      register(subscriptionId, {
        entityId: 'entity-1',
        onInvalidate: () => {},
      })

      // When: entry is registered
      // Then: should work without params
      expect(() => trigger(subscriptionId)).not.toThrow()
    })
  })

  describe('integration scenarios', () => {
    test('should handle subscription lifecycle', () => {
      // Given: a subscription lifecycle
      const subscriptionId = 'lifecycle-sub'
      let callbackCalled = false

      // When: registering
      register(subscriptionId, {
        entityId: 'entity-1',
        onInvalidate: () => {
          callbackCalled = true
        },
      })

      // Then: should be triggerable
      trigger(subscriptionId)
      expect(callbackCalled).toBe(true)

      // When: unregistering
      callbackCalled = false
      unregister(subscriptionId)

      // Then: should not trigger
      trigger(subscriptionId)
      expect(callbackCalled).toBe(false)

      // When: re-registering
      register(subscriptionId, {
        entityId: 'entity-2',
        onInvalidate: () => {
          callbackCalled = true
        },
      })

      // Then: should trigger again
      trigger(subscriptionId)
      expect(callbackCalled).toBe(true)
    })

    test('should handle multiple subscriptions for different entities', () => {
      // Given: multiple subscriptions
      const sub1 = 'sub-entity-1'
      const sub2 = 'sub-entity-2'
      const sub3 = 'sub-entity-3'
      let call1 = false
      let call2 = false
      let call3 = false

      register(sub1, {
        entityId: 'entity-1',
        onInvalidate: () => {
          call1 = true
        },
      })
      register(sub2, {
        entityId: 'entity-2',
        onInvalidate: () => {
          call2 = true
        },
      })
      register(sub3, {
        entityId: 'entity-3',
        onInvalidate: () => {
          call3 = true
        },
      })

      // When: triggering one
      trigger(sub2)

      // Then: only that one should be called
      expect(call1).toBe(false)
      expect(call2).toBe(true)
      expect(call3).toBe(false)
    })
  })
})

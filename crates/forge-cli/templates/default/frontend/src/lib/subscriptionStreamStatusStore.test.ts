/**
 * BDD tests for subscriptionStreamStatusStore
 * Tests verify store exports, initial state, get/update, and layer behavior.
 */
import { describe, test, expect } from 'bun:test'
import { Effect } from 'effect'
import {
  SubscriptionStreamStatusStore,
  SubscriptionStreamStatusStoreTag,
  subscriptionStreamStatusStoreLayer,
  initialSubscriptionStreamStatus,
  getSubscriptionStreamStatusStoreLayer,
} from './subscriptionStreamStatusStore'

describe('subscriptionStreamStatusStore', () => {
  describe('exports', () => {
    test('should export SubscriptionStreamStatusStore with tag and layer', () => {
      // Given: the store module
      // When: checking the store definition
      // Then: should have tag and layer
      expect(SubscriptionStreamStatusStore).toBeDefined()
      expect(SubscriptionStreamStatusStore.tag).toBeDefined()
      expect(SubscriptionStreamStatusStore.layer).toBeDefined()
    })

    test('should export SubscriptionStreamStatusStoreTag', () => {
      // Given: the store module
      // When: checking the tag export
      // Then: should be defined
      expect(SubscriptionStreamStatusStoreTag).toBeDefined()
    })

    test('should export subscriptionStreamStatusStoreLayer', () => {
      // Given: the store module
      // When: checking the layer export
      // Then: should be defined
      expect(subscriptionStreamStatusStoreLayer).toBeDefined()
    })

    test('should export initialSubscriptionStreamStatus', () => {
      // Given: the store module
      // When: checking the initial state
      // Then: should be { connected: false, connectionId: null }
      expect(initialSubscriptionStreamStatus).toEqual({
        connected: false,
        connectionId: null,
      })
    })

    test('should export getSubscriptionStreamStatusStoreLayer', () => {
      // Given: the store module
      // When: checking the layer getter
      // Then: should be a function returning the layer
      expect(typeof getSubscriptionStreamStatusStoreLayer).toBe('function')
      expect(getSubscriptionStreamStatusStoreLayer()).toBe(
        subscriptionStreamStatusStoreLayer,
      )
    })
  })

  describe('initial state', () => {
    test('should initialize with connected false', async () => {
      // Given: the store layer
      // When: getting the initial value
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(subscriptionStreamStatusStoreLayer)),
      )

      // Then: connected should be false, connectionId null
      expect(result).toEqual({ connected: false, connectionId: null })
    })
  })

  describe('get behavior', () => {
    test('should return current status', async () => {
      // Given: the store layer
      // When: getting value without updates
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(subscriptionStreamStatusStoreLayer)),
      )

      // Then: should return initial status
      expect(result.connected).toBe(false)
    })

    test('should return updated status after update', async () => {
      // Given: the store layer
      // When: setting connected to true then getting
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        yield* store.update((prev) => ({ ...prev, connected: true }))
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(subscriptionStreamStatusStoreLayer)),
      )

      // Then: connected should be true
      expect(result.connected).toBe(true)
    })
  })

  describe('update behavior', () => {
    test('should update connected to true', async () => {
      // Given: the store layer
      // When: updating connected to true
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        yield* store.update((prev) => ({ ...prev, connected: true }))
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(subscriptionStreamStatusStoreLayer)),
      )

      // Then: status should have connected true
      expect(result.connected).toBe(true)
    })

    test('should update connected back to false', async () => {
      // Given: the store layer
      // When: setting connected true then false
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        yield* store.update((prev) => ({ ...prev, connected: true }))
        yield* store.update((prev) => ({ ...prev, connected: false }))
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(subscriptionStreamStatusStoreLayer)),
      )

      // Then: connected should be false
      expect(result.connected).toBe(false)
    })

    test('should preserve ReactiveStore update contract', async () => {
      // Given: the store layer
      // When: updating with a function that receives previous state
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        yield* store.update((prev) => ({ ...prev, connected: !prev.connected }))
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(subscriptionStreamStatusStoreLayer)),
      )

      // Then: connected should be toggled from initial false to true
      expect(result.connected).toBe(true)
    })
  })

  describe('layer behavior', () => {
    test('should provide store via subscriptionStreamStatusStoreLayer', async () => {
      // Given: subscriptionStreamStatusStoreLayer
      // When: running an effect that requires the store
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(subscriptionStreamStatusStoreLayer)),
      )

      // Then: should get status with connected boolean (store is shared across tests)
      expect(result).toHaveProperty('connected')
      expect(typeof result.connected).toBe('boolean')
    })

    test('should provide store via getSubscriptionStreamStatusStoreLayer', async () => {
      // Given: layer from getter
      const layer = getSubscriptionStreamStatusStoreLayer()

      // When: running an effect with that layer
      const program = Effect.gen(function* () {
        const store = yield* SubscriptionStreamStatusStoreTag
        return yield* store.get()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )

      // Then: should get status with connected boolean (store is shared across tests)
      expect(result).toHaveProperty('connected')
      expect(typeof result.connected).toBe('boolean')
    })
  })
})

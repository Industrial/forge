/**
 * BDD tests for SubscriptionStream service interface.
 * Tests verify the behavior of the SubscriptionStreamService contract through SubscriptionStreamMock.
 */
import { describe, test, expect } from 'bun:test'
import { Chunk, Effect, Stream } from 'effect'
import { SubscriptionStreamMock } from './SubscriptionStreamMock'
import {
  SubscriptionStream,
  type SubscriptionStreamEvent,
} from './SubscriptionStream'

describe('SubscriptionStream service', () => {
  describe('openStream behavior', () => {
    test('should return a stream successfully', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        return yield* service.openStream()
      })

      const stream = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should return a Stream (check by trying to consume it)
      expect(stream).toBeDefined()
    })

    test('should emit ready event', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream and collecting events
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(stream)
      })

      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should emit ready event
      const events = Chunk.toReadonlyArray(chunk)
      expect(events).toHaveLength(1)
      expect(events[0]).toEqual({ type: 'ready', connection_id: 'mock-conn' })
    })

    test('should return stream that emits SubscriptionStreamEvent', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(stream)
      })

      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: events should match SubscriptionStreamEvent type
      const events = Chunk.toReadonlyArray(chunk)
      expect(events).toHaveLength(1)
      const event = events[0] as SubscriptionStreamEvent
      expect(event).toHaveProperty('type')
      expect('type' in event && event.type).toBe('ready')
    })

    test('should always succeed when opening stream', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        return yield* service.openStream()
      })

      // Then: should not throw
      const stream = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      expect(stream).toBeDefined()
    })

    test('should return stream that can be consumed multiple times', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream and consuming it multiple times
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()

        // Consume stream first time
        const chunk1 = yield* Stream.runCollect(stream)
        const events1 = Chunk.toReadonlyArray(chunk1)

        // Open new stream for second consumption
        const stream2 = yield* service.openStream()
        const chunk2 = yield* Stream.runCollect(stream2)
        const events2 = Chunk.toReadonlyArray(chunk2)

        return { events1, events2 }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: both consumptions should work
      expect(result.events1).toHaveLength(1)
      expect(result.events2).toHaveLength(1)
      expect(result.events1[0]).toEqual({
        type: 'ready',
        connection_id: 'mock-conn',
      })
      expect(result.events2[0]).toEqual({
        type: 'ready',
        connection_id: 'mock-conn',
      })
    })
  })

  describe('stream events behavior', () => {
    test('should emit ready event as first event', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream and taking first event
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(Stream.take(stream, 1))
      })

      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: first event should be ready
      const events = Chunk.toReadonlyArray(chunk)
      expect(events).toHaveLength(1)
      expect(events[0]).toEqual({ type: 'ready', connection_id: 'mock-conn' })
    })

    test('should emit ready event with correct structure', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(stream)
      })

      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: ready event should have correct structure
      const events = Chunk.toReadonlyArray(chunk)
      expect(events[0]).toEqual({ type: 'ready', connection_id: 'mock-conn' })
      expect((events[0] as { type: string }).type).toBe('ready')
    })
  })

  describe('Layer behavior', () => {
    test('should provide SubscriptionStream service', async () => {
      // Given: SubscriptionStreamMock layer
      // When: accessing SubscriptionStream
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        return service
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should return SubscriptionStreamService
      expect(result).toBeDefined()
      expect(result.openStream).toBeDefined()
      expect(typeof result.openStream).toBe('function')
    })

    test('should not require any dependencies', async () => {
      // Given: SubscriptionStreamMock layer (no dependencies)
      // When: using it directly
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(stream)
      })

      // Then: should work without providing additional layers
      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      const events = Chunk.toReadonlyArray(chunk)
      expect(events).toHaveLength(1)
      expect(events[0]).toEqual({ type: 'ready', connection_id: 'mock-conn' })
    })

    test('should create new stream instance for each openStream call', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream multiple times
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream1 = yield* service.openStream()
        const stream2 = yield* service.openStream()
        const chunk1 = yield* Stream.runCollect(stream1)
        const chunk2 = yield* Stream.runCollect(stream2)
        const events1 = Chunk.toReadonlyArray(chunk1)
        const events2 = Chunk.toReadonlyArray(chunk2)
        return { events1, events2 }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: each stream should work independently
      expect(result.events1).toHaveLength(1)
      expect(result.events2).toHaveLength(1)
      expect(result.events1[0]).toEqual({
        type: 'ready',
        connection_id: 'mock-conn',
      })
      expect(result.events2[0]).toEqual({
        type: 'ready',
        connection_id: 'mock-conn',
      })
    })
  })

  describe('stream consumption behavior', () => {
    test('should allow consuming stream with runCollect', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream and collecting all events
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(stream)
      })

      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should collect all events
      const events = Chunk.toReadonlyArray(chunk)
      expect(events).toHaveLength(1)
      expect(events[0]).toEqual({ type: 'ready', connection_id: 'mock-conn' })
    })

    test('should allow consuming stream with take', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream and taking first event
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(Stream.take(stream, 1))
      })

      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should take first event
      const events = Chunk.toReadonlyArray(chunk)
      expect(events).toHaveLength(1)
      expect(events[0]).toEqual({ type: 'ready', connection_id: 'mock-conn' })
    })

    test('should allow consuming stream with runForEach', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream and iterating events
      const collected: SubscriptionStreamEvent[] = []
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runForEach(stream, (event) =>
          Effect.sync(() => {
            collected.push(event)
          }),
        )
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should collect events via runForEach
      expect(collected).toHaveLength(1)
      expect(collected[0]).toEqual({
        type: 'ready',
        connection_id: 'mock-conn',
      })
    })
  })

  describe('service interface contract', () => {
    test('should implement SubscriptionStreamService interface', async () => {
      // Given: SubscriptionStreamMock layer
      // When: accessing service
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        return service
      })

      const service = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should have openStream method
      expect(service).toHaveProperty('openStream')
      expect(typeof service.openStream).toBe('function')
    })

    test('should return Effect that resolves to Stream', async () => {
      // Given: SubscriptionStreamMock layer
      // When: calling openStream
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const streamEffect = service.openStream()
        return yield* streamEffect
      })

      const stream = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: should return Stream (check by trying to consume it)
      expect(stream).toBeDefined()
    })

    test('should return stream that emits SubscriptionStreamEvent', async () => {
      // Given: SubscriptionStreamMock layer
      // When: opening stream and collecting events
      const program = Effect.gen(function* () {
        const service = yield* SubscriptionStream
        const stream = yield* service.openStream()
        return yield* Stream.runCollect(stream)
      })

      const chunk = await Effect.runPromise(
        program.pipe(Effect.provide(SubscriptionStreamMock)),
      )

      // Then: events should match SubscriptionStreamEvent type
      const events = Chunk.toReadonlyArray(chunk)
      expect(events.length).toBeGreaterThan(0)
      const event = events[0] as SubscriptionStreamEvent
      // Event should be either { type: 'ready', connection_id } or { subscription_id: string }
      expect(
        ('type' in event && event.type === 'ready') ||
          typeof (event as { subscription_id?: string }).subscription_id ===
            'string',
      ).toBe(true)
    })
  })
})

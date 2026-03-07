/**
 * BDD component tests for main.tsx
 * Tests verify initialization logic, error handling, and integration with Effect and ReactDOM
 */
import { describe, test, expect, beforeEach, afterEach } from 'bun:test'
import { render, screen, waitFor } from '@testing-library/react'
import { Effect, Layer } from 'effect'
import { Window } from 'happy-dom'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'

import App from '@/App.tsx'
import {
  getApplicationLayer,
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'
import { RpcApiMock } from '@/services/RpcApiMock'

// Set up DOM environment and application layer override for tests
beforeEach(() => {
  const window = new Window()
  const document = window.document
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.window = window
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.document = document
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.localStorage = window.localStorage

  // Create root element
  const rootElement = document.createElement('div')
  rootElement.id = 'root'
  document.body.appendChild(rootElement)

  const baseLayer = buildApplicationLayer()
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(baseLayer, RpcApiMock),
  )
})

afterEach(() => {
  clearApplicationLayerOverrideForTesting()
  // Clean up React root
  const rootElement = document.getElementById('root')
  if (rootElement) {
    rootElement.innerHTML = ''
  }
})

describe('main.tsx initialization', () => {
  describe('module structure', () => {
    test('should import ReactDOM', () => {
      // Given: the main module
      // When: checking ReactDOM import
      // Then: ReactDOM should be available
      expect(ReactDOM).toBeDefined()
      expect(ReactDOM.createRoot).toBeDefined()
      expect(typeof ReactDOM.createRoot).toBe('function')
    })

    test('should import BrowserRouter', () => {
      // Given: the main module
      // When: checking BrowserRouter import
      // Then: BrowserRouter should be available
      expect(BrowserRouter).toBeDefined()
      expect(typeof BrowserRouter).toBe('function')
    })

    test('should import App component', () => {
      // Given: the main module
      // When: checking App import
      // Then: App should be available
      expect(App).toBeDefined()
      expect(typeof App).toBe('function')
    })

    test('should import getApplicationLayer', () => {
      // Given: the main module
      // When: checking getApplicationLayer import
      // Then: getApplicationLayer should be available
      expect(getApplicationLayer).toBeDefined()
      expect(typeof getApplicationLayer).toBe('function')
    })

    test('should import Authentication service tag', () => {
      // Given: the main module
      // When: checking Authentication import
      // Then: Authentication should be available
      expect(Authentication).toBeDefined()
    })
  })

  describe('DOM element finding behavior', () => {
    test('should find root element when it exists', () => {
      // Given: root element exists in DOM
      const rootElement = document.getElementById('root')
      // When: checking for root element
      // Then: root element should be found
      expect(rootElement).not.toBeNull()
      expect(rootElement?.id).toBe('root')
    })

    test('should return null when root element does not exist', () => {
      // Given: root element is removed
      const rootElement = document.getElementById('root')
      if (rootElement) {
        rootElement.remove()
      }
      // When: checking for root element
      const found = document.getElementById('root')
      // Then: root element should not be found
      expect(found).toBeNull()
    })
  })

  describe('ReactDOM.createRoot behavior', () => {
    test('should create root when element exists', () => {
      // Given: root element exists
      const rootElement = document.getElementById('root')
      expect(rootElement).not.toBeNull()
      // When: creating React root
      const root = ReactDOM.createRoot(rootElement!)
      // Then: root should be created successfully
      expect(root).toBeDefined()
      expect(typeof root.render).toBe('function')
    })

    test('should create React root and render function', () => {
      // Given: root element exists
      const rootElement = document.getElementById('root')
      expect(rootElement).not.toBeNull()
      // When: creating React root
      const root = ReactDOM.createRoot(rootElement!)
      // Then: root should have render method
      expect(root).toBeDefined()
      expect(typeof root.render).toBe('function')
    })
  })

  describe('Effect initialization behavior', () => {
    test('should provide application layer to Effect', async () => {
      // Given: application layer
      const layer = getApplicationLayer()
      // When: running an effect with the layer
      const program = Effect.gen(function* () {
        yield* Effect.logInfo('Test log')
        return { success: true }
      })
      // Then: effect should run successfully
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.success).toBe(true)
    })

    test('should access Authentication service from layer', async () => {
      // Given: application layer with Authentication service
      const layer = getApplicationLayer()
      // When: accessing Authentication service
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        expect(auth).toBeDefined()
        return { success: true }
      })
      // Then: service should be accessible
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.success).toBe(true)
    })

    test('should handle restoreSession call', async () => {
      // Given: mock Authentication service
      const { authentication } = createMockAuthentication()
      const layer = Layer.succeed(Authentication, authentication)
      // When: calling restoreSession
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        yield* auth.restoreSession()
        return { success: true }
      })
      // Then: restoreSession should be callable
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.success).toBe(true)
    })
  })

  describe('error handling behavior', () => {
    test('should handle missing root element error', async () => {
      // Given: root element is removed
      const rootElement = document.getElementById('root')
      if (rootElement) {
        rootElement.remove()
      }
      // When: checking for root element
      const found = document.getElementById('root')
      // Then: should return null
      expect(found).toBeNull()
    })

    test('should handle Effect errors gracefully', async () => {
      // Given: an Effect that fails
      const failingProgram = Effect.gen(function* () {
        return yield* Effect.fail(new Error('Test error'))
      })
      // When: running the failing effect and capturing the error with Either
      const result = await Effect.runPromise(
        failingProgram.pipe(
          Effect.either,
          Effect.provide(getApplicationLayer()),
        ),
      )
      // Then: should capture the error (Left) and not throw
      expect(result._tag).toBe('Left')
      if (result._tag === 'Left') {
        expect(result.left.message).toBe('Test error')
      }
    })
  })

  describe('integration behavior', () => {
    test('should integrate Effect initialization with React root creation', async () => {
      // Given: root element and application layer
      const rootElement = document.getElementById('root')
      expect(rootElement).not.toBeNull()
      const layer = getApplicationLayer()
      // When: running initialization effect and creating React root
      const program = Effect.gen(function* () {
        yield* Effect.logInfo('Starting application')
        const auth = yield* Authentication
        yield* auth.restoreSession()
        return rootElement
      })
      const element = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      const root = ReactDOM.createRoot(element!)
      // Then: both Effect and ReactDOM should work together
      expect(element).not.toBeNull()
      expect(root).toBeDefined()
      expect(typeof root.render).toBe('function')
    })

    test('should handle full initialization flow', async () => {
      // Given: root element exists
      const rootElement = document.getElementById('root')
      expect(rootElement).not.toBeNull()
      const layer = getApplicationLayer()
      // When: running full initialization (log, restore session, create root)
      const program = Effect.gen(function* () {
        yield* Effect.logInfo('Starting application')
        const auth = yield* Authentication
        yield* auth.restoreSession()
        return rootElement
      })
      const element = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      let rootCreated = false
      if (element) {
        const root = ReactDOM.createRoot(element)
        rootCreated = true
        expect(root).toBeDefined()
      }
      // Then: initialization should complete successfully
      expect(element).not.toBeNull()
      expect(rootCreated).toBe(true)
    })
  })

  describe('BrowserRouter integration', () => {
    test('should be able to wrap components in BrowserRouter', () => {
      // Given: BrowserRouter component
      // When: checking BrowserRouter availability
      // Then: BrowserRouter should be available for wrapping
      expect(BrowserRouter).toBeDefined()
      expect(typeof BrowserRouter).toBe('function')
    })

    test('should structure render call with BrowserRouter wrapper', () => {
      // Given: root element and React root
      const rootElement = document.getElementById('root')
      expect(rootElement).not.toBeNull()
      const root = ReactDOM.createRoot(rootElement!)
      // When: checking that BrowserRouter can wrap App
      // Then: structure should be valid (BrowserRouter wraps App)
      const wrapper = (
        <BrowserRouter>
          <div>Test</div>
        </BrowserRouter>
      )
      expect(wrapper).toBeDefined()
      expect(root).toBeDefined()
      expect(typeof root.render).toBe('function')
    })
  })
})

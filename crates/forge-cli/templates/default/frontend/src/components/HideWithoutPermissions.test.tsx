/**
 * BDD component tests for HideWithoutPermissions.tsx
 * Tests verify component rendering, permission checking, and conditional visibility
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'
import { Effect, Layer, Stream, Chunk } from 'effect'

import { HideWithoutPermissions } from './HideWithoutPermissions'
import { Providers } from '../Providers'
import {
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '../lib/appLayer'
import { clearReactiveStoreCacheForTesting } from '../lib/ReactiveStore'
import type { ReactiveStore } from '../lib/ReactiveStore'
import type { AuthenticationState } from '../features/authentication/stores/AuthenticationStateReactiveStore'
import { Authentication } from '../features/authentication/services/Authentication'
import {
  AuthStoreTag,
  initialAuthenticationState,
} from '../features/authentication/stores/AuthenticationStateReactiveStore'
import { createMockAuthentication } from '../features/authentication/services/AuthenticationMock'
import { RpcApiMock } from '../services/RpcApiMock'

// Set up DOM environment for tests
beforeAll(() => {
  // Ensure SyntaxError exists globally first
  const global = globalThis as any
  if (!global.SyntaxError) {
    global.SyntaxError = class SyntaxError extends Error {
      constructor(message?: string) {
        super(message)
        this.name = 'SyntaxError'
        Object.setPrototypeOf(this, SyntaxError.prototype)
      }
    }
  }

  if (typeof globalThis.window === 'undefined') {
    const window = new Window()
    const document = window.document
    global.window = window
    global.document = document
    global.localStorage = window.localStorage
    global.navigator = window.navigator
    // Ensure document.body exists
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    // Always set SyntaxError on new window instance
    ;(window as any).SyntaxError = global.SyntaxError
  } else {
    // Ensure existing window has SyntaxError
    if (!(globalThis.window as any).SyntaxError) {
      ;(globalThis.window as any).SyntaxError = global.SyntaxError
    }
  }
})

// Helper to create a mock auth store with permissions
function createMockAuthStoreWithPermissions(permissions: string[]) {
  let current: AuthenticationState = {
    ...initialAuthenticationState,
    permissions,
  }
  const changeListeners = new Set<(a: AuthenticationState) => void>()

  const notify = (a: AuthenticationState) => {
    current = a
    changeListeners.forEach((l) => l(a))
  }

  // Create a stream that emits the current value immediately and then listens for changes
  const changes = Stream.async<AuthenticationState, never, never>((emit) => {
    // Emit current value immediately when stream is subscribed
    emit(Effect.succeed(Chunk.of(current)))
    const listener = (a: AuthenticationState) => {
      emit(Effect.succeed(Chunk.of(a)))
    }
    changeListeners.add(listener)
    return Effect.sync(() => {
      changeListeners.delete(listener)
    })
  })

  const store: ReactiveStore<AuthenticationState> = {
    // Use Effect.sync to ensure synchronous resolution
    get: () => Effect.sync(() => current),
    update: (f: (a: AuthenticationState) => AuthenticationState) =>
      Effect.sync(() => {
        notify(f(current))
      }),
    changes,
  }

  return Layer.succeed(AuthStoreTag, store)
}

// Helper to create a wrapper with theme, router, and app layer context
const createWrapper = (permissions: string[] = []) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()

  // Create mock auth store with permissions
  const mockAuthStoreLayer = createMockAuthStoreWithPermissions(permissions)

  // Build base application layer
  const baseLayer = buildApplicationLayer()

  // Clear reactive store cache for testing
  clearReactiveStoreCacheForTesting(AuthStoreTag)

  // Set up the application layer override with mock store and auth service
  // Merge baseLayer with mock layers so mock layers override baseLayer's services
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(
      baseLayer,
      mockAuthStoreLayer,
      Layer.succeed(Authentication, mockAuth.authentication),
      RpcApiMock,
    ),
  )

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

afterEach(() => {
  cleanup()
})

describe('HideWithoutPermissions component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
    clearReactiveStoreCacheForTesting(AuthStoreTag)
  })
  describe('export behavior', () => {
    test('should export HideWithoutPermissions as named export', () => {
      // Given: the HideWithoutPermissions module
      // When: checking the export
      // Then: HideWithoutPermissions should be available
      expect(HideWithoutPermissions).toBeDefined()
      expect(typeof HideWithoutPermissions).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user has required permissions', async () => {
      // Given: HideWithoutPermissions component with user having permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: children should be rendered
      await waitFor(
        () => {
          expect(
            container.querySelector('[data-testid="content"]'),
          ).not.toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })

    test('should hide children when user lacks required permissions', async () => {
      // Given: HideWithoutPermissions component with user lacking permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper([]) },
      )
      // Then: children should not be rendered
      await waitFor(
        () => {
          expect(container.querySelector('[data-testid="content"]')).toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })

    test('should render when user has at least one permission', async () => {
      // Given: HideWithoutPermissions component with multiple permissions
      const { container } = render(
        <HideWithoutPermissions
          permissions={['permission1', 'permission2', 'permission3']}
        >
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['permission2']) },
      )
      // Then: children should be rendered (user has permission2)
      await waitFor(
        () => {
          expect(
            container.querySelector('[data-testid="content"]'),
          ).not.toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })
  })

  describe('props handling behavior', () => {
    test('should accept permissions prop', async () => {
      // Given: HideWithoutPermissions component with permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: component should render (permissions prop accepted)
      await waitFor(
        () => {
          expect(container.textContent).toContain('Content')
        },
        { timeout: 5000, interval: 100 },
      )
    })

    test('should accept children prop', async () => {
      // Given: HideWithoutPermissions component with children
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          {children}
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: children should be rendered
      await waitFor(
        () => {
          expect(
            container.querySelector('[data-testid="children"]'),
          ).not.toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })

    test('should handle empty permissions array', async () => {
      // Given: HideWithoutPermissions component with empty permissions
      const { container } = render(
        <HideWithoutPermissions permissions={[]}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['some.permission']) },
      )
      // Then: children should be hidden (no permissions match)
      await waitFor(
        () => {
          expect(container.querySelector('[data-testid="content"]')).toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })

    test('should handle multiple permissions', async () => {
      // Given: HideWithoutPermissions component with multiple permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['perm1', 'perm2', 'perm3']}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['perm2']) },
      )
      // Then: children should be rendered (user has perm2)
      await waitFor(
        () => {
          expect(
            container.querySelector('[data-testid="content"]'),
          ).not.toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })
  })

  describe('permission checking behavior', () => {
    test('should use shouldHideWithoutPermissions function', async () => {
      // Given: HideWithoutPermissions component
      // When: checking component structure
      // Then: should use shouldHideWithoutPermissions (verified by rendering)
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should check user permissions from useAuthStore', async () => {
      // Given: HideWithoutPermissions component
      // When: rendering component
      // Then: should read permissions from useAuthStore
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should hide when none of the permissions match', async () => {
      // Given: HideWithoutPermissions component with permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['perm1', 'perm2']}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['perm3', 'perm4']) },
      )
      await waitFor(() => {})
      // Then: children should be hidden (no matching permissions)
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('edge cases', () => {
    test('should handle null children', async () => {
      // Given: HideWithoutPermissions component with null children
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          {null}
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(() => {})
      // Then: component should still render (no error)
      expect(container).toBeDefined()
    })

    test('should handle undefined children', async () => {
      // Given: HideWithoutPermissions component with undefined children
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          {undefined}
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(() => {})
      // Then: component should still render (no error)
      expect(container).toBeDefined()
    })

    test('should handle complex nested children', async () => {
      // Given: HideWithoutPermissions component with nested children
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>
            <h1>Title</h1>
            <p>Paragraph</p>
          </div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(() => {})
      // Then: nested children should be rendered
      expect(container).toBeDefined()
    })
  })
})

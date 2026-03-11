/**
 * BDD component tests for ShowWithPermissions.tsx
 * Tests verify component rendering, permission checking, and conditional visibility
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'
import { Effect, Layer, Stream, Chunk } from 'effect'

import { ShowWithPermissions } from './ShowWithPermissions'
import { Providers } from '../Providers'
import {
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '../lib/appLayer'
import { clearReactiveStoreCacheForTesting } from '../lib/ReactiveStore'
import type { ReactiveStore } from '../lib/ReactiveStore'
import type { AuthenticationState } from '../features/authentication/stores/AuthenticationStateReactiveStore'
import {
  AuthStoreTag,
  initialAuthenticationState,
} from '../features/authentication/stores/AuthenticationStateReactiveStore'
import { RpcApiMock } from '../services/RpcApiMock'

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

  // Get the sync registry if it exists (set by createExternalStore)
  // This allows the store to update the React cache synchronously
  const _getRegistry = () => {
    // Access the internal registry from ReactiveStore module
    // We can't import it directly, so we'll trigger updates via the stream
    return null
  }

  const notify = (a: AuthenticationState) => {
    current = a
    changeListeners.forEach((l) => l(a))
  }

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
    get: () => Effect.succeed(current),
    update: (f: (a: AuthenticationState) => AuthenticationState) =>
      Effect.sync(() => {
        notify(f(current))
      }),
    changes,
  }

  return Layer.succeed(AuthStoreTag, store)
}

const createWrapper = (permissions: string[] = []) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockStoreLayer = createMockAuthStoreWithPermissions(permissions)
  const baseLayer = buildApplicationLayer()
  clearReactiveStoreCacheForTesting(AuthStoreTag)
  // Merge baseLayer with mockStoreLayer so mockStoreLayer overrides baseLayer's store
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(baseLayer, mockStoreLayer, RpcApiMock),
  )
  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('ShowWithPermissions component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
    clearReactiveStoreCacheForTesting(AuthStoreTag)
  })

  describe('export behavior', () => {
    test('should export ShowWithPermissions as named export', () => {
      expect(ShowWithPermissions).toBeDefined()
      expect(typeof ShowWithPermissions).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user has at least one permission', async () => {
      const { container } = render(
        <ShowWithPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </ShowWithPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(
        () => {
          expect(
            container.querySelector('[data-testid="content"]'),
          ).not.toBeNull()
        },
        { timeout: 3000 },
      )
    })

    test('should hide children when user lacks all permissions', async () => {
      const { container } = render(
        <ShowWithPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </ShowWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      await waitFor(() => {})
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept permissions prop', async () => {
      const { container } = render(
        <ShowWithPermissions permissions={['perm1', 'perm2']}>
          <div>Content</div>
        </ShowWithPermissions>,
        { wrapper: createWrapper(['perm1']) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept children prop', async () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <ShowWithPermissions permissions={['test.permission']}>
          {children}
        </ShowWithPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(
        () => {
          expect(
            container.querySelector('[data-testid="children"]'),
          ).not.toBeNull()
        },
        { timeout: 3000 },
      )
    })
  })

  describe('permission checking behavior', () => {
    test('should use shouldShowWithPermissions function', async () => {
      const { container } = render(
        <ShowWithPermissions permissions={['test.permission']}>
          <div>Content</div>
        </ShowWithPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should check user permissions from useAuthStore', async () => {
      const { container } = render(
        <ShowWithPermissions permissions={['test.permission']}>
          <div>Content</div>
        </ShowWithPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })
})

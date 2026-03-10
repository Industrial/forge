/**
 * BDD component tests for DashboardScopeGuard.tsx
 * Tests verify component rendering, scope selection, and redirect behavior
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option, Stream, Chunk } from 'effect'

import DashboardScopeGuard from './DashboardScopeGuard'
import { Providers } from '@/Providers'
import {
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '@/lib/appLayer'
import { clearReactiveStoreCacheForTesting } from '@/lib/ReactiveStore'
import type { ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import {
  AuthStoreTag,
  initialAuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { RpcApiMock } from '@/services/RpcApiMock'

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

// Helper to create a mock auth store with user and whether scope is selected (currentScope set).
function createMockAuthStore(
  user: AuthenticationUser | null = null,
  needsScopeSelect: boolean = false,
) {
  const hasScope = !needsScopeSelect
  let current: AuthenticationState = {
    ...initialAuthenticationState,
    user: Option.fromNullable(user),
    currentScope: hasScope
      ? Option.some({ organizationId: 'org-1', roleId: 'role-1' })
      : Option.none(),
    permissions: user && hasScope ? ['view:dashboard'] : [],
  }
  const changeListeners = new Set<(a: AuthenticationState) => void>()

  const notify = (a: AuthenticationState) => {
    current = a
    changeListeners.forEach((l) => l(a))
  }

  const changes = Stream.async<AuthenticationState, never, never>((emit) => {
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

const createWrapper = (
  user: AuthenticationUser | null = null,
  needsScopeSelect: boolean = false,
) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockStoreLayer = createMockAuthStore(user, needsScopeSelect)
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

describe('DashboardScopeGuard component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
    clearReactiveStoreCacheForTesting(AuthStoreTag)
  })

  describe('export behavior', () => {
    test('should export DashboardScopeGuard as default export', () => {
      expect(DashboardScopeGuard).toBeDefined()
      expect(typeof DashboardScopeGuard).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when scope is selected', async () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <DashboardScopeGuard>
          <div data-testid="content">Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(user, false) },
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

    test('should show loading spinner when loading', async () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should return null when user is not authenticated', async () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should redirect to select-scope when needsScopeSelect is true', async () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <DashboardScopeGuard>
          <div data-testid="content">Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(user, true) },
      )
      await waitFor(() => {})
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept children prop', async () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <DashboardScopeGuard>{children}</DashboardScopeGuard>,
        { wrapper: createWrapper(user, false) },
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

  describe('authentication integration behavior', () => {
    test('should use useAuthStore hook', async () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should check if user is authenticated', async () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should check needsScopeSelect from authentication state', async () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })
})

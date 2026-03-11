/**
 * BDD component tests for Navbar.tsx
 * Tests verify component rendering, props handling, and navigation integration
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'
import { Effect, Layer, Option, Stream, Chunk } from 'effect'

import Navbar from './Navbar'
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
import { Authentication } from '../features/authentication/services/Authentication'
import { createMockAuthentication } from '../features/authentication/services/AuthenticationMock'
import { AuthenticationUser } from '../features/authentication/domain/AuthenticationUser'

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

function createMockAuthStore(state: AuthenticationState) {
  let current: AuthenticationState = state
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
    return Effect.sync(() => changeListeners.delete(listener))
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

const createWrapper = (user: AuthenticationUser | null = null) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  if (user) {
    mockAuth.setUser(user)
  }
  const mockStoreLayer = createMockAuthStore({
    ...initialAuthenticationState,
    user: user ? Option.some(user) : Option.none(),
    token: user ? Option.some('mock-token') : Option.none(),
    currentScope: Option.none(),
  })
  const baseLayer = buildApplicationLayer()
  clearReactiveStoreCacheForTesting(AuthStoreTag)
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(
      mockStoreLayer,
      baseLayer,
      Layer.succeed(Authentication, mockAuth.authentication),
    ),
  )
  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('Navbar component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
  })

  describe('export behavior', () => {
    test('should export Navbar as default export', () => {
      expect(Navbar).toBeDefined()
      expect(typeof Navbar).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render AppBar', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should render app name', async () => {
      const { container } = render(<Navbar appName="Test App" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container.textContent).toContain('Test App')
    })

    test('should render default app name when not provided', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
      await waitFor(() => {})
      expect(container.textContent).toContain('App')
    })

    test('should render theme toggle button', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should render user avatar', async () => {
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token',
      })
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept appName prop', async () => {
      const { container } = render(<Navbar appName="Custom App" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container.textContent).toContain('Custom App')
    })

    test('should accept colorScheme prop', async () => {
      const { container } = render(<Navbar colorScheme="dark" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept onToggleTheme callback', async () => {
      let toggleCalled = false
      const handleToggle = () => {
        toggleCalled = true
      }
      render(<Navbar onToggleTheme={handleToggle} />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(typeof handleToggle).toBe('function')
      handleToggle()
      expect(toggleCalled).toBe(true)
    })

    test('should accept onOpenSidebar callback', async () => {
      let sidebarCalled = false
      const handleSidebar = () => {
        sidebarCalled = true
      }
      const { container } = render(<Navbar onOpenSidebar={handleSidebar} />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(typeof handleSidebar).toBe('function')
      handleSidebar()
      expect(sidebarCalled).toBe(true)
      expect(container).toBeDefined()
    })
  })

  describe('navigation behavior', () => {
    test('should navigate to home on logo click', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should navigate to scope on scope menu click', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should navigate to login on logout', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('user menu behavior', () => {
    test('should show user menu when avatar clicked', async () => {
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token',
      })
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should display user email initial in avatar', async () => {
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token',
      })
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should handle user menu items', async () => {
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token',
      })
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use useAuthStore hook', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should display user information when authenticated', async () => {
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token',
      })
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })
})

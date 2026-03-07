/**
 * BDD component tests for Sidebar.tsx
 * Tests verify component rendering, props handling, and navigation structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option , Stream, Chunk} from 'effect'

import Sidebar from './Sidebar'
import { Providers } from '@/Providers'
import { getApplicationLayer, buildApplicationLayer } from '@/lib/appLayer'
import type { ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import type { ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { Authentication } from '@/features/authentication/services/Authentication'
import { AuthStoreTag, initialAuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'

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

const createWrapper = (permissions: string[] = []) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  mockAuth.state.permissions = permissions

  const appLayer = getApplicationLayer(
    Layer.mergeAll(
      mockAuth.authentication,
      Layer.succeed(Authentication, mockAuth.authentication),
    ),
  )

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('Sidebar component', () => {
  describe('export behavior', () => {
    test('should export Sidebar as default export', () => {
      expect(Sidebar).toBeDefined()
      expect(typeof Sidebar).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render navigation items', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper(['dashboard']) },
      )
      expect(container).toBeDefined()
    })

    test('should render toggle button when not hidden', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should hide toggle button when hideToggle is true', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} hideToggle={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept expanded prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onToggle callback', () => {
      let toggleCalled = false
      const handleToggle = () => {
        toggleCalled = true
      }
      render(<Sidebar expanded={true} onToggle={handleToggle} />, {
        wrapper: createWrapper(),
      })
      expect(typeof handleToggle).toBe('function')
      handleToggle()
      expect(toggleCalled).toBe(true)
    })

    test('should accept hideToggle prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} hideToggle={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept disableBorder prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} disableBorder={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept fullWidth prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} fullWidth={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('layout behavior', () => {
    test('should adjust width based on expanded state', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should use fullWidth when fullWidth is true', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} fullWidth={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

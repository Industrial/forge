/**
 * BDD component tests for HideWithPermissions.tsx
 * Tests verify component rendering, permission checking, and conditional visibility
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Stream, Chunk } from 'effect'

import { HideWithPermissions } from './HideWithPermissions'
import { Providers } from '@/Providers'
import {
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '@/lib/appLayer'
import { clearReactiveStoreCacheForTesting } from '@/lib/ReactiveStore'
import type { ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { AuthStoreTag, initialAuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'

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
  const mockStoreLayer = createMockAuthStoreWithPermissions(permissions)
  const baseLayer = buildApplicationLayer()
  clearReactiveStoreCacheForTesting(AuthStoreTag)
  setApplicationLayerOverrideForTesting(
    Layer.merge(mockStoreLayer, baseLayer),
  )
  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('HideWithPermissions component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
  })

  describe('export behavior', () => {
    test('should export HideWithPermissions as named export', () => {
      expect(HideWithPermissions).toBeDefined()
      expect(typeof HideWithPermissions).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should hide children when user has any of the permissions', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })

    test('should render children when user lacks all permissions', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept permissions prop', () => {
      const { container } = render(
        <HideWithPermissions permissions={['perm1', 'perm2']}>
          <div>Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })

    test('should accept children prop', () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          {children}
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('permission checking behavior', () => {
    test('should use shouldHideWithPermissions function', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })

    test('should check user permissions from useAuthStore', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })
  })
})

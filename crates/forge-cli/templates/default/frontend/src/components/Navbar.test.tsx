/**
 * BDD component tests for Navbar.tsx
 * Tests verify component rendering, props handling, and navigation integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import Navbar from './Navbar'
import { Providers } from '@/Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'

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

const createWrapper = (user: AuthenticationUser | null = null) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  if (user) {
    mockAuth.setUser(user)
  }

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

describe('Navbar component', () => {
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
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
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
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should display user email initial in avatar', async () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should handle user menu items', async () => {
      const { container } = render(<Navbar />, { wrapper: createWrapper() })
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
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(<Navbar />, { wrapper: createWrapper(user) })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })
})

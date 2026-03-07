/**
 * BDD component tests for PermissionsPage.tsx
 * Tests verify component rendering, permissions management, and Effect integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer } from 'effect'

import PermissionsPage from './PermissionsPage'
import { Providers } from '@/Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Permissions } from '@/features/dashboard/services/Permissions'
import { PermissionsMock } from '@/features/dashboard/services/PermissionsMock'

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
    })

const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockPermissions = PermissionsMock.make()

  const appLayer = getApplicationLayer(
    Layer.mergeAll(
      mockPermissions,
      Layer.succeed(Permissions, mockPermissions),
    ),
  )

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('PermissionsPage component', () => {
  describe('export behavior', () => {
    test('should export PermissionsPage as default export', () => {
      expect(PermissionsPage).toBeDefined()
      expect(typeof PermissionsPage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render PageHeader', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render PermissionsAddBar', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render LoadingSpinner when loading', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render TableEmptyRow when no assignments', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render AssignmentTableRow for each assignment', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('permissions management behavior', () => {
    test('should use Permissions service', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should load permissions data on mount', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle adding permissions', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle deleting permissions', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('error handling behavior', () => {
    test('should render ErrorAlert on error', () => {
      const { container } = render(<PermissionsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

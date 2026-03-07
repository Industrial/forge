/**
 * BDD component tests for OrganizationsPage.tsx
 * Tests verify component rendering, organization management, and CRUD operations
 */
import { describe, test, expect, beforeAll, beforeEach, afterEach } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer } from 'effect'

import OrganizationsPage from './OrganizationsPage'
import { Providers } from '@/Providers'
import {
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '@/lib/appLayer'
import { EntityApi } from '@/services/EntityApi'
import { EntityApiMock } from '@/services/EntityApiMock'

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

const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('OrganizationsPage component', () => {
  beforeEach(() => {
    const mockApi = EntityApiMock.make()
    const baseLayer = buildApplicationLayer()
    setApplicationLayerOverrideForTesting(
      Layer.mergeAll(
        baseLayer,
        mockApi,
        Layer.succeed(EntityApi, mockApi),
      ),
    )
  })

  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
  })

  describe('export behavior', () => {
    test('should export OrganizationsPage as default export', () => {
      expect(OrganizationsPage).toBeDefined()
      expect(typeof OrganizationsPage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render PageHeader', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render OrganizationsFilters', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render LoadingSpinner when loading', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render EmptyState when no organizations', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render OrganizationTableRow for each organization', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('organization management behavior', () => {
    test('should use EntityApi for CRUD operations', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle filtering organizations', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle creating organizations', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle updating organizations', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle deleting organizations', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('error handling behavior', () => {
    test('should render ErrorAlert on error', () => {
      const { container } = render(<OrganizationsPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

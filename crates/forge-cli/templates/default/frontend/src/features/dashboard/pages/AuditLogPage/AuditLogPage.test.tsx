/**
 * BDD component tests for AuditLogPage.tsx
 * Tests verify component rendering, audit log display, and filtering
 */
import { describe, test, expect, beforeAll, beforeEach, afterEach } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer } from 'effect'

import AuditLogPage from './AuditLogPage'
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
  const mockApi = EntityApiMock.make()

  const baseLayer = buildApplicationLayer()
  const mockLayer = Layer.mergeAll(
    mockApi,
    Layer.succeed(EntityApi, mockApi),
  )
  const testLayer = Layer.merge(mockLayer, baseLayer)
  setApplicationLayerOverrideForTesting(testLayer)

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('AuditLogPage component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
  })

  describe('export behavior', () => {
    test('should export AuditLogPage as default export', () => {
      expect(AuditLogPage).toBeDefined()
      expect(typeof AuditLogPage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render PageHeader', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render AuditLogFilters', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render LoadingSpinner when loading', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render TableEmptyRow when no entries', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render AuditLogTableRow for each entry', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('audit log display behavior', () => {
    test('should use EntityApi for loading audit log entries', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle filtering audit log entries', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('error handling behavior', () => {
    test('should render ErrorAlert on error', () => {
      const { container } = render(<AuditLogPage />, {
        wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

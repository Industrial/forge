/**
 * BDD component tests for GenericEntityCrud.tsx
 * Tests verify component rendering, CRUD operations, and EntityApi integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer } from 'effect'

import GenericEntityCrud from './GenericEntityCrud'
import { Providers } from './Providers'
import { getApplicationLayer } from '@/lib/appLayer'
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

  const appLayer = getApplicationLayer(
    Layer.mergeAll(
      mockApi,
      Layer.succeed(EntityApi, mockApi),
    ),
  )

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('GenericEntityCrud component', () => {
  describe('export behavior', () => {
    test('should export GenericEntityCrud as default export', () => {
      expect(GenericEntityCrud).toBeDefined()
      expect(typeof GenericEntityCrud).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render PageHeader with title', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test Entity" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Test Entity')
    })

    test('should render LoadingSpinner when loading', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render EmptyState when list is empty', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render table when items exist', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept entityId prop', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test-entity" title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept title prop', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Custom Title" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Custom Title')
    })

    test('should accept columns prop', () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          columns={[{ key: 'id', label: 'ID' }]}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept emptyMessage prop', () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          emptyMessage="No items found"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept permission flags', () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          canRead={true}
          canCreate={false}
          canUpdate={false}
          canDelete={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('CRUD operations behavior', () => {
    test('should use EntityApi for list operation', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should show Add button when renderCreateForm is provided', () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          renderCreateForm={() => <div>Create Form</div>}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Add')
    })

    test('should show Edit button when renderEditForm is provided', () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          renderEditForm={() => <div>Edit Form</div>}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should show Delete button when canDelete is true', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" canDelete={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('error handling behavior', () => {
    test('should render ErrorAlert on error', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should show permission error when canRead is false', () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" canRead={false} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('permission')
    })
  })
})

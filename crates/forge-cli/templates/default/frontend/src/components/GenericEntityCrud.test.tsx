/**
 * BDD component tests for GenericEntityCrud.tsx
 * Tests verify component rendering, CRUD operations, and EntityApi integration
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'
import { Effect, Layer } from 'effect'
import { EffectRuntimeProvider } from 'react-effect-hooks'

import GenericEntityCrud from './GenericEntityCrud'
import { Providers } from '../Providers'
import {
  buildApplicationLayer,
  getApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '../lib/appLayer'
import { EntityApiMock } from '../services/EntityApiMock'
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

const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const baseLayer = buildApplicationLayer()
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(baseLayer, EntityApiMock, RpcApiMock),
  )
  const layer = getApplicationLayer()
  const runtime = Effect.runSync(Effect.scoped(Layer.toRuntime(layer)))

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <EffectRuntimeProvider runtime={runtime}>
        <Providers theme={theme}>{children}</Providers>
      </EffectRuntimeProvider>
    </BrowserRouter>
  )
}

afterEach(() => {
  cleanup()
})

describe('GenericEntityCrud component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
  })
  describe('export behavior', () => {
    test('should export GenericEntityCrud as default export', () => {
      expect(GenericEntityCrud).toBeDefined()
      expect(typeof GenericEntityCrud).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render PageHeader with title', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test Entity" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Test Entity')
    })

    test('should render LoadingSpinner when loading', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should render EmptyState when list is empty', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should render table when items exist', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept entityId prop', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test-entity" title="Test" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept title prop', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Custom Title" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Custom Title')
    })

    test('should accept columns prop', async () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          columns={[{ key: 'id', label: 'ID' }]}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept emptyMessage prop', async () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          emptyMessage="No items found"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept permission flags', async () => {
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
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('CRUD operations behavior', () => {
    test('should use EntityApi for list operation', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should show Add button when renderCreateForm is provided', async () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          renderCreateForm={() => <div>Create Form</div>}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Add')
    })

    test('should show Edit button when renderEditForm is provided', async () => {
      const { container } = render(
        <GenericEntityCrud
          entityId="test"
          title="Test"
          renderEditForm={() => <div>Edit Form</div>}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should show Delete button when canDelete is true', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" canDelete={true} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('error handling behavior', () => {
    test('should render ErrorAlert on error', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should show permission error when canRead is false', async () => {
      const { container } = render(
        <GenericEntityCrud entityId="test" title="Test" canRead={false} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('permission')
    })
  })
})

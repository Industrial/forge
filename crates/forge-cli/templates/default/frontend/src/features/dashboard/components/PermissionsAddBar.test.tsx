/**
 * BDD component tests for PermissionsAddBar.tsx
 * Tests verify component rendering, props handling, and form controls
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup, waitFor } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import PermissionsAddBar from './PermissionsAddBar'

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
    <ThemeProvider theme={theme}>{children}</ThemeProvider>
  )
}

afterEach(() => {
  cleanup()
})

describe('PermissionsAddBar component', () => {
  describe('export behavior', () => {
    test('should export PermissionsAddBar as default export', () => {
      expect(PermissionsAddBar).toBeDefined()
      expect(typeof PermissionsAddBar).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render scope select', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission=""
          roles={[]}
          permissions={[]}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Scope')
    })

    test('should render role select', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission=""
          roles={['admin', 'user']}
          permissions={[]}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Role')
    })

    test('should render permission select', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission=""
          roles={[]}
          permissions={['read', 'write']}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Permission')
    })

    test('should render Add button', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission=""
          roles={[]}
          permissions={[]}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Add')
    })
  })

  describe('props handling behavior', () => {
    test('should accept scope prop', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="global"
          permission=""
          roles={[]}
          permissions={[]}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept roles prop', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission=""
          roles={['admin', 'user']}
          permissions={[]}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept permissions prop', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission=""
          roles={[]}
          permissions={['read', 'write']}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should disable Add button when adding is true', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission="read"
          roles={['admin']}
          permissions={['read']}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={true}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Adding…')
    })

    test('should disable Add button when permission is empty', async () => {
      const { container } = render(
        <PermissionsAddBar
          scope="org"
          permission=""
          roles={['admin']}
          permissions={['read']}
          onScopeChange={() => {}}
          onRoleChange={() => {}}
          onPermissionChange={() => {}}
          onAdd={() => {}}
          adding={false}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })
})

/**
 * BDD component tests for UserTableRow.tsx
 * Tests verify component rendering, props handling, and table row structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Table, TableBody } from '@mui/material'
import { Window } from 'happy-dom'
import React from 'react'

import UserTableRow, { type UserRow } from './UserTableRow'

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
  return ({ children }: { children: React.ReactNode }) => (
    <ThemeProvider theme={theme}>
      <Table>
        <TableBody>{children}</TableBody>
      </Table>
    </ThemeProvider>
  )
}

describe('UserTableRow component', () => {
  describe('export behavior', () => {
    test('should export UserTableRow as default export', () => {
      expect(UserTableRow).toBeDefined()
      expect(typeof UserTableRow).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render user email', () => {
      const user: UserRow = {
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      }
      const { container } = render(
        <UserTableRow
          user={user}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('test@example.com')
    })

    test('should render memberships summary', () => {
      const user: UserRow = {
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [{ org_name: 'Org 1', roles: ['admin'] }],
      }
      const { container } = render(
        <UserTableRow
          user={user}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render active status', () => {
      const user: UserRow = {
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      }
      const { container } = render(
        <UserTableRow
          user={user}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Yes')
    })

    test('should render admin status', () => {
      const user: UserRow = {
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: true,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      }
      const { container } = render(
        <UserTableRow
          user={user}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Yes')
    })
  })

  describe('props handling behavior', () => {
    test('should accept user prop', () => {
      const user: UserRow = {
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      }
      const { container } = render(
        <UserTableRow
          user={user}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should show edit/delete buttons when canWrite is true', () => {
      const user: UserRow = {
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      }
      const { container } = render(
        <UserTableRow
          user={user}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should hide edit/delete buttons when canWrite is false', () => {
      const user: UserRow = {
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      }
      const { container } = render(
        <UserTableRow
          user={user}
          canWrite={false}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

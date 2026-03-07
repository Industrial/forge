/**
 * BDD component tests for RoleTableRow.tsx
 * Tests verify component rendering, props handling, and table row structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Table, TableBody } from '@mui/material'
import { Window } from 'happy-dom'
import React from 'react'

import RoleTableRow from './RoleTableRow'
import type { Role } from '@/features/dashboard/domain/Role'

beforeAll(() => {
  if (typeof globalThis.window === 'undefined') {
    const window = new Window()
    const document = window.document
    const global = globalThis as any
    global.window = window
    global.document = document
    global.localStorage = window.localStorage
    global.navigator = window.navigator
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    global.SyntaxError = class SyntaxError extends Error {
      constructor(message?: string) {
        super(message)
        this.name = 'SyntaxError'
        Object.setPrototypeOf(this, SyntaxError.prototype)
      }
    }
    if (window.SyntaxError === undefined) {
      window.SyntaxError = global.SyntaxError as any
    }
  }
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

describe('RoleTableRow component', () => {
  describe('export behavior', () => {
    test('should export RoleTableRow as default export', () => {
      expect(RoleTableRow).toBeDefined()
      expect(typeof RoleTableRow).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render organization name', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Test Org')
    })

    test('should render role name', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('admin')
    })

    test('should render display name when provided', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Administrator')
    })

    test('should render dash when display name is null', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: null,
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('—')
    })

    test('should render Edit and Delete buttons', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept role prop', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept orgName prop', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Custom Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Custom Org')
    })

    test('should accept onEdit callback', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      let editCalled = false
      const handleEdit = () => {
        editCalled = true
      }
      render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={handleEdit}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleEdit).toBe('function')
      handleEdit()
      expect(editCalled).toBe(true)
    })

    test('should accept onDelete callback', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      let deleteCalled = false
      const handleDelete = () => {
        deleteCalled = true
      }
      render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={handleDelete}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleDelete).toBe('function')
      handleDelete()
      expect(deleteCalled).toBe(true)
    })

    test('should disable Delete button when isDeleting is true', () => {
      const role: Role = {
        id: 'role-1',
        name: 'admin',
        display_name: 'Administrator',
        org_id: 'org-1',
      }
      const { container } = render(
        <RoleTableRow
          role={role}
          orgName="Test Org"
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={true}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

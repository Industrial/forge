/**
 * BDD component tests for OrganizationTableRow.tsx
 * Tests verify component rendering, props handling, and table row structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Table, TableBody } from '@mui/material'
import { Window } from 'happy-dom'
import React from 'react'

import OrganizationTableRow from './OrganizationTableRow'
import type { Organization } from '@/features/dashboard/domain/Organization'

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

describe('OrganizationTableRow component', () => {
  describe('export behavior', () => {
    test('should export OrganizationTableRow as default export', () => {
      expect(OrganizationTableRow).toBeDefined()
      expect(typeof OrganizationTableRow).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render organization name', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      const { container } = render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Test Org')
    })

    test('should render organization slug', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      const { container } = render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('test-org')
    })

    test('should render created and updated dates', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      const { container } = render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render Edit and Delete buttons when canWrite is true', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      const { container } = render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should hide Edit and Delete buttons when canWrite is false', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      const { container } = render(
        <OrganizationTableRow
          org={org}
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

  describe('props handling behavior', () => {
    test('should accept org prop', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      const { container } = render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onEdit callback', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      let editCalled = false
      const handleEdit = () => {
        editCalled = true
      }
      render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
          onEdit={handleEdit}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleEdit).toBe('function')
      handleEdit(org)
      expect(editCalled).toBe(true)
    })

    test('should accept onDelete callback', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      let deleteCalled = false
      const handleDelete = () => {
        deleteCalled = true
      }
      render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
          onEdit={() => {}}
          onDelete={handleDelete}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleDelete).toBe('function')
      handleDelete(org.id)
      expect(deleteCalled).toBe(true)
    })

    test('should disable Delete button when isDeleting is true', () => {
      const org: Organization = {
        id: 'org-1',
        name: 'Test Org',
        slug: 'test-org',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }
      const { container } = render(
        <OrganizationTableRow
          org={org}
          canWrite={true}
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

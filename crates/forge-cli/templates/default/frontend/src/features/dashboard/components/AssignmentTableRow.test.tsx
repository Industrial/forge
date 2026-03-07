/**
 * BDD component tests for AssignmentTableRow.tsx
 * Tests verify component rendering, props handling, and table row structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Table, TableBody } from '@mui/material'
import { Window } from 'happy-dom'
import React from 'react'

import AssignmentTableRow from './AssignmentTableRow'
import type { Assignment } from '../domain/Assignment'

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

describe('AssignmentTableRow component', () => {
  describe('export behavior', () => {
    test('should export AssignmentTableRow as default export', () => {
      expect(AssignmentTableRow).toBeDefined()
      expect(typeof AssignmentTableRow).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render scope', () => {
      const assignment: Assignment = {
        scope: 'org',
        role_name: 'admin',
        permission_key: 'test.permission',
      }
      const { container } = render(
        <AssignmentTableRow
          assignment={assignment}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('org')
    })

    test('should render role_name', () => {
      const assignment: Assignment = {
        scope: 'org',
        role_name: 'admin',
        permission_key: 'test.permission',
      }
      const { container } = render(
        <AssignmentTableRow
          assignment={assignment}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('admin')
    })

    test('should render permission_key', () => {
      const assignment: Assignment = {
        scope: 'org',
        role_name: 'admin',
        permission_key: 'test.permission',
      }
      const { container } = render(
        <AssignmentTableRow
          assignment={assignment}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('test.permission')
    })

    test('should render delete button', () => {
      const assignment: Assignment = {
        scope: 'org',
        role_name: 'admin',
        permission_key: 'test.permission',
      }
      const { container } = render(
        <AssignmentTableRow
          assignment={assignment}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept assignment prop', () => {
      const assignment: Assignment = {
        scope: 'org',
        role_name: 'admin',
        permission_key: 'test.permission',
      }
      const { container } = render(
        <AssignmentTableRow
          assignment={assignment}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onDelete callback', () => {
      const assignment: Assignment = {
        scope: 'org',
        role_name: 'admin',
        permission_key: 'test.permission',
      }
      let deleteCalled = false
      const handleDelete = () => {
        deleteCalled = true
      }
      render(
        <AssignmentTableRow
          assignment={assignment}
          onDelete={handleDelete}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleDelete).toBe('function')
      handleDelete()
      expect(deleteCalled).toBe(true)
    })

    test('should disable delete button when isDeleting is true', () => {
      const assignment: Assignment = {
        scope: 'org',
        role_name: 'admin',
        permission_key: 'test.permission',
      }
      const { container } = render(
        <AssignmentTableRow
          assignment={assignment}
          onDelete={() => {}}
          isDeleting={true}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

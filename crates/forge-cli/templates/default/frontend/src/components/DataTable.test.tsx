/**
 * BDD component tests for DataTable.tsx
 * Tests verify component rendering, columns, loading state, and pagination
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import DataTable from './DataTable'
import type { DataTableColumn, DataTablePagination } from './DataTable'

beforeAll(() => {
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
    ;(window as any).SyntaxError = global.SyntaxError
  } else {
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

type Row = { id: string; name: string }

const columns: DataTableColumn<Row>[] = [
  { id: 'name', label: 'Name', render: (row) => row.name },
]
const pagination: DataTablePagination = {
  page: 0,
  rowsPerPage: 10,
  totalCount: 0,
  onPageChange: () => {},
  onRowsPerPageChange: () => {},
  rowsPerPageOptions: [10, 25, 50],
}

describe('DataTable component', () => {
  describe('export behavior', () => {
    test('should export DataTable as default export', () => {
      expect(DataTable).toBeDefined()
      expect(typeof DataTable).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render table with column headers', () => {
      const { getByText } = render(
        <DataTable
          columns={columns}
          rows={[]}
          pagination={pagination}
          emptyMessage="No items"
        />,
        { wrapper: createWrapper() },
      )
      expect(getByText('Name')).toBeTruthy()
    })

    test('should show empty message when rows are empty', () => {
      const { getByText } = render(
        <DataTable
          columns={columns}
          rows={[]}
          pagination={pagination}
          emptyMessage="No items"
        />,
        { wrapper: createWrapper() },
      )
      expect(getByText('No items')).toBeTruthy()
    })

    test('should render rows when data provided', () => {
      const { getByText } = render(
        <DataTable
          columns={columns}
          rows={[{ id: '1', name: 'Alice' }]}
          pagination={pagination}
          emptyMessage="No items"
        />,
        { wrapper: createWrapper() },
      )
      expect(getByText('Alice')).toBeTruthy()
    })

    test('should show loading spinner when loading is true', () => {
      const { container } = render(
        <DataTable
          columns={columns}
          rows={[]}
          loading={true}
          pagination={pagination}
          emptyMessage="No items"
        />,
        { wrapper: createWrapper() },
      )
      expect(container.innerHTML).toContain('svg')
    })
  })

  describe('pagination behavior', () => {
    test('should render pagination controls', () => {
      const { getByLabelText } = render(
        <DataTable
          columns={columns}
          rows={[]}
          pagination={pagination}
          emptyMessage="No items"
        />,
        { wrapper: createWrapper() },
      )
      expect(getByLabelText(/rows per page/i)).toBeTruthy()
    })
  })
})

/**
 * BDD component tests for EntityTableRow.tsx
 * Tests verify row rendering, columns, and edit/delete actions
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import EntityTableRow from './EntityTableRow'
import type { EntityTableRowColumn } from './EntityTableRow'

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

type Item = { id: string; name: string }
const columns: EntityTableRowColumn<Item>[] = [
  { key: 'name', render: (item) => item.name },
]

afterEach(() => {
  cleanup()
})

describe('EntityTableRow component', () => {
  describe('export behavior', () => {
    test('should export EntityTableRow as default export', () => {
      expect(EntityTableRow).toBeDefined()
      expect(typeof EntityTableRow).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render columns for item', () => {
      const { getByText } = render(
        <table>
          <tbody>
            <EntityTableRow
              item={{ id: '1', name: 'Test' }}
              columns={columns}
              getRowId={(item) => item.id}
            />
          </tbody>
        </table>,
        { wrapper: createWrapper() },
      )
      expect(getByText('Test')).toBeTruthy()
    })

    test('should use testIdPrefix for row test id', () => {
      const { getByTestId } = render(
        <table>
          <tbody>
            <EntityTableRow
              item={{ id: 'user-1', name: 'Alice' }}
              columns={columns}
              getRowId={(item) => item.id}
              testIdPrefix="user-row"
            />
          </tbody>
        </table>,
        { wrapper: createWrapper() },
      )
      expect(getByTestId('user-row-user-1')).toBeTruthy()
    })

    test('should render Edit and Delete buttons when canEditDelete and handlers provided', () => {
      const { getByLabelText } = render(
        <table>
          <tbody>
            <EntityTableRow
              item={{ id: '1', name: 'Item' }}
              columns={columns}
              getRowId={(item) => item.id}
              canEditDelete
              onEdit={() => {}}
              onDelete={() => {}}
            />
          </tbody>
        </table>,
        { wrapper: createWrapper() },
      )
      expect(getByLabelText('Edit')).toBeTruthy()
      expect(getByLabelText('Delete')).toBeTruthy()
    })
  })
})

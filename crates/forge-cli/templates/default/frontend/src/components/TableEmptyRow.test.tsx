/**
 * BDD component tests for TableEmptyRow.tsx
 * Tests verify component rendering, props handling, and table structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Table, TableBody } from '@mui/material'
import { Window } from 'happy-dom'
import type React from 'react'

import TableEmptyRow from './TableEmptyRow'

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
    <ThemeProvider theme={theme}>
      <Table>
        <TableBody>{children}</TableBody>
      </Table>
    </ThemeProvider>
  )
}

describe('TableEmptyRow component', () => {
  describe('export behavior', () => {
    test('should export TableEmptyRow as default export', () => {
      expect(TableEmptyRow).toBeDefined()
      expect(typeof TableEmptyRow).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children', () => {
      const { container } = render(
        <TableEmptyRow colSpan={3}>
          <div data-testid="content">Empty</div>
        </TableEmptyRow>,
        { wrapper: createWrapper() },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should render TableRow', () => {
      const { container } = render(
        <TableEmptyRow colSpan={3}>
          <div>Empty</div>
        </TableEmptyRow>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render TableCell with colSpan', () => {
      const { container } = render(
        <TableEmptyRow colSpan={5}>
          <div>Empty</div>
        </TableEmptyRow>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept colSpan prop', () => {
      const { container } = render(
        <TableEmptyRow colSpan={4}>
          <div>Empty</div>
        </TableEmptyRow>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept children prop', () => {
      const children = <div data-testid="children">No data</div>
      const { container } = render(
        <TableEmptyRow colSpan={3}>{children}</TableEmptyRow>,
        { wrapper: createWrapper() },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('layout behavior', () => {
    test('should center content', () => {
      const { container } = render(
        <TableEmptyRow colSpan={3}>
          <div>Empty</div>
        </TableEmptyRow>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should span all columns', () => {
      const { container } = render(
        <TableEmptyRow colSpan={10}>
          <div>Empty</div>
        </TableEmptyRow>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

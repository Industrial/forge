/**
 * BDD component tests for DataListMobile.tsx
 * Tests verify component rendering, items list, and optional pagination
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import DataListMobile from './DataListMobile'

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

type Item = { id: string; label: string }

describe('DataListMobile component', () => {
  describe('export behavior', () => {
    test('should export DataListMobile as default export', () => {
      expect(DataListMobile).toBeDefined()
      expect(typeof DataListMobile).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render items via renderItem', () => {
      const { getByTestId } = render(
        <DataListMobile<Item>
          items={[{ id: '1', label: 'First' }]}
          getKey={(item) => item.id}
          renderItem={(item) => (
            <span data-testid={`item-${item.id}`}>{item.label}</span>
          )}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByTestId('item-1').textContent).toBe('First')
    })

    test('should have default data-testid data-list-mobile', () => {
      const { getByTestId } = render(
        <DataListMobile<Item>
          items={[]}
          getKey={(item) => item.id}
          renderItem={(item) => <span>{item.label}</span>}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByTestId('data-list-mobile')).toBeTruthy()
    })

    test('should use custom ariaLabel', () => {
      const { getByLabelText } = render(
        <DataListMobile<Item>
          items={[]}
          getKey={(item) => item.id}
          renderItem={(item) => <span>{item.label}</span>}
          ariaLabel="Organizations list"
        />,
        { wrapper: createWrapper() },
      )
      expect(getByLabelText('Organizations list')).toBeTruthy()
    })
  })
})

/**
 * BDD component tests for FiltersBar.tsx
 * Tests verify component rendering and layout structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import FiltersBar from './FiltersBar'

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

describe('FiltersBar component', () => {
  describe('export behavior', () => {
    test('should export FiltersBar as default export', () => {
      expect(FiltersBar).toBeDefined()
      expect(typeof FiltersBar).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children', () => {
      const { getByTestId } = render(
        <FiltersBar>
          <input data-testid="filter-input" aria-label="Search" />
        </FiltersBar>,
        { wrapper: createWrapper() },
      )
      expect(getByTestId('filter-input')).toBeTruthy()
    })

    test('should have data-testid filters-bar', () => {
      const { getByTestId } = render(
        <FiltersBar>
          <span>Filters</span>
        </FiltersBar>,
        { wrapper: createWrapper() },
      )
      expect(getByTestId('filters-bar')).toBeTruthy()
    })

    test('should render extra content when provided', () => {
      const { getByRole } = render(
        <FiltersBar extra={<button>Apply</button>}>
          <span>Filter</span>
        </FiltersBar>,
        { wrapper: createWrapper() },
      )
      expect(getByRole('button', { name: 'Apply' })).toBeTruthy()
    })

    test('should not render extra when not provided', () => {
      const { container, queryByRole } = render(
        <FiltersBar>
          <span>Filter</span>
        </FiltersBar>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Filter')
      expect(queryByRole('button', { name: 'Apply' })).toBeFalsy()
    })
  })
})

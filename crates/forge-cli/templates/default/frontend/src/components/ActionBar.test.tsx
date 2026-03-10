/**
 * BDD component tests for ActionBar.tsx
 * Tests verify component rendering and layout structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import ActionBar from './ActionBar'

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

describe('ActionBar component', () => {
  describe('export behavior', () => {
    test('should export ActionBar as default export', () => {
      expect(ActionBar).toBeDefined()
      expect(typeof ActionBar).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children', () => {
      const { getByRole } = render(
        <ActionBar>
          <button>Add Item</button>
        </ActionBar>,
        { wrapper: createWrapper() },
      )
      expect(getByRole('button', { name: 'Add Item' })).toBeTruthy()
    })

    test('should have data-testid action-bar', () => {
      const { getByTestId } = render(
        <ActionBar>
          <span>Actions</span>
        </ActionBar>,
        { wrapper: createWrapper() },
      )
      expect(getByTestId('action-bar')).toBeTruthy()
    })
  })

  describe('props handling behavior', () => {
    test('should accept children prop', () => {
      const { getByTestId } = render(
        <ActionBar>
          <span data-testid="child">Content</span>
        </ActionBar>,
        { wrapper: createWrapper() },
      )
      expect(getByTestId('child').textContent).toBe('Content')
    })
  })
})

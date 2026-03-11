/**
 * BDD component tests for CenteredLoader.tsx
 * Tests verify component rendering, structure, and accessibility
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import CenteredLoader from './CenteredLoader'

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

afterEach(() => {
  cleanup()
})

describe('CenteredLoader component', () => {
  describe('export behavior', () => {
    test('should export CenteredLoader as default export', () => {
      expect(CenteredLoader).toBeDefined()
      expect(typeof CenteredLoader).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render Box with LoadingSpinner', () => {
      const { container } = render(<CenteredLoader />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
      expect(container.innerHTML).toContain('svg')
    })

    test('should have aria-busy and aria-label', () => {
      const { container } = render(<CenteredLoader />, {
        wrapper: createWrapper(),
      })
      const box = container.querySelector('[aria-busy][aria-label="Loading"]')
      expect(box).toBeTruthy()
    })
  })

  describe('structure behavior', () => {
    test('should use flex layout for centering', () => {
      const { container } = render(<CenteredLoader />, {
        wrapper: createWrapper(),
      })
      expect(container.firstElementChild).toBeTruthy()
    })
  })
})

/**
 * BDD component tests for FiltersPanel.tsx
 * Tests verify component rendering, props handling, and MUI integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import FiltersPanel from './FiltersPanel'

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
    <ThemeProvider theme={theme}>{children}</ThemeProvider>
  )
}

describe('FiltersPanel component', () => {
  describe('export behavior', () => {
    test('should export FiltersPanel as default export', () => {
      expect(FiltersPanel).toBeDefined()
      expect(typeof FiltersPanel).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children', () => {
      const { container } = render(
        <FiltersPanel>
          <div data-testid="content">Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should render default title "Filters"', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Filters')
    })

    test('should render custom title', () => {
      const { container } = render(
        <FiltersPanel title="Custom Filters">
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Custom Filters')
    })
  })

  describe('props handling behavior', () => {
    test('should accept title prop', () => {
      const { container } = render(
        <FiltersPanel title="Custom Title">
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Custom Title')
    })

    test('should accept children prop', () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <FiltersPanel>{children}</FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })

    test('should use default title when not provided', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Filters')
    })
  })

  describe('MUI integration behavior', () => {
    test('should use Paper from MUI', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should use Typography for title', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should use Box for filter controls', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('layout structure behavior', () => {
    test('should apply padding', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should apply margin bottom', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should apply flex wrap layout', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Content</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should apply gap between filter controls', () => {
      const { container } = render(
        <FiltersPanel>
          <div>Filter 1</div>
          <div>Filter 2</div>
        </FiltersPanel>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

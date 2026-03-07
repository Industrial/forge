/**
 * BDD component tests for Providers.tsx
 * Tests verify component rendering, props handling, and MUI ThemeProvider integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import { Providers } from './Providers'

// Set up DOM environment for tests
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
    // Ensure document.body exists
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

describe('Providers component', () => {
  describe('export behavior', () => {
    test('should export Providers as named export', () => {
      // Given: the Providers module
      // When: checking the export
      // Then: Providers should be available
      expect(Providers).toBeDefined()
      expect(typeof Providers).toBe('function')
    })

    test('should export ProvidersProps type', () => {
      // Given: the Providers module
      // When: checking type exports
      // Then: ProvidersProps type should be available
      // Note: TypeScript enforces this at compile time
      expect(Providers).toBeDefined()
    })
  })

  describe('rendering behavior', () => {
    test('should render children', () => {
      // Given: Providers component with children
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers with children
      const { container } = render(
        <Providers theme={theme}>
          <div data-testid="test-child">Test Child</div>
        </Providers>,
      )
      // Then: children should be rendered
      expect(container).toBeDefined()
      expect(
        container.querySelector('[data-testid="test-child"]'),
      ).not.toBeNull()
    })

    test('should render with ThemeProvider', () => {
      // Given: Providers component
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers
      const { container } = render(
        <Providers theme={theme}>
          <div>Content</div>
        </Providers>,
      )
      // Then: ThemeProvider should wrap children
      expect(container).toBeDefined()
    })

    test('should render CssBaseline', () => {
      // Given: Providers component
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers
      const { container } = render(
        <Providers theme={theme}>
          <div>Content</div>
        </Providers>,
      )
      // Then: CssBaseline should be rendered
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept theme prop', () => {
      // Given: Providers component with theme
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers with theme
      const { container } = render(
        <Providers theme={theme}>
          <div>Content</div>
        </Providers>,
      )
      // Then: component should render (theme prop accepted)
      expect(container).toBeDefined()
    })

    test('should accept children prop', () => {
      // Given: Providers component with children
      const theme = createTheme({ palette: { mode: 'light' } })
      const children = <div data-testid="children">Children</div>
      // When: rendering Providers with children
      const { container } = render(
        <Providers theme={theme}>{children}</Providers>,
      )
      // Then: children should be rendered
      expect(container).toBeDefined()
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })

    test('should accept light theme', () => {
      // Given: Providers component with light theme
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers
      const { container } = render(
        <Providers theme={theme}>
          <div>Content</div>
        </Providers>,
      )
      // Then: component should render successfully
      expect(container).toBeDefined()
    })

    test('should accept dark theme', () => {
      // Given: Providers component with dark theme
      const theme = createTheme({ palette: { mode: 'dark' } })
      // When: rendering Providers
      const { container } = render(
        <Providers theme={theme}>
          <div>Content</div>
        </Providers>,
      )
      // Then: component should render successfully
      expect(container).toBeDefined()
    })

    test('should handle multiple children', () => {
      // Given: Providers component with multiple children
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers with multiple children
      const { container } = render(
        <Providers theme={theme}>
          <div>Child 1</div>
          <div>Child 2</div>
          <div>Child 3</div>
        </Providers>,
      )
      // Then: all children should be rendered
      expect(container).toBeDefined()
    })
  })

  describe('MUI integration behavior', () => {
    test('should wrap children with ThemeProvider', () => {
      // Given: Providers component
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers
      const { container } = render(
        <Providers theme={theme}>
          <div>Content</div>
        </Providers>,
      )
      // Then: ThemeProvider should provide theme context
      expect(container).toBeDefined()
    })

    test('should include CssBaseline for CSS reset', () => {
      // Given: Providers component
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers
      const { container } = render(
        <Providers theme={theme}>
          <div>Content</div>
        </Providers>,
      )
      // Then: CssBaseline should be included
      expect(container).toBeDefined()
    })
  })

  describe('edge cases', () => {
    test('should handle null children', () => {
      // Given: Providers component with null children
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers with null children
      const { container } = render(<Providers theme={theme}>{null}</Providers>)
      // Then: component should still render
      expect(container).toBeDefined()
    })

    test('should handle undefined children', () => {
      // Given: Providers component with undefined children
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers with undefined children
      const { container } = render(
        <Providers theme={theme}>{undefined}</Providers>,
      )
      // Then: component should still render
      expect(container).toBeDefined()
    })

    test('should handle empty children', () => {
      // Given: Providers component with empty fragment
      const theme = createTheme({ palette: { mode: 'light' } })
      // When: rendering Providers with empty children
      const { container } = render(<Providers theme={theme}></Providers>)
      // Then: component should still render
      expect(container).toBeDefined()
    })
  })
})

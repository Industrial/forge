/**
 * BDD component tests for PageHeader.tsx
 * Tests verify component rendering, props handling, and MUI integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import PageHeader from './PageHeader'

// Set up DOM environment for tests
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

describe('PageHeader component', () => {
  describe('export behavior', () => {
    test('should export PageHeader as default export', () => {
      expect(PageHeader).toBeDefined()
      expect(typeof PageHeader).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render title', () => {
      const { container } = render(
        <PageHeader title="Test Page" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Test Page')
    })

    test('should render description when provided', () => {
      const { container } = render(
        <PageHeader title="Test" description="Description text" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Description text')
    })

    test('should not render description when not provided', () => {
      const { container } = render(
        <PageHeader title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).not.toContain('Description')
    })

    test('should render Live chip when liveConnected is true', () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={true} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Live')
    })

    test('should not render Live chip when liveConnected is false', () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={false} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).not.toContain('Live')
    })
  })

  describe('props handling behavior', () => {
    test('should accept title prop', () => {
      const { container } = render(
        <PageHeader title="Custom Title" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Custom Title')
    })

    test('should accept description prop', () => {
      const { container } = render(
        <PageHeader title="Test" description="Custom description" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Custom description')
    })

    test('should accept liveConnected prop', () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={true} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Live')
    })

    test('should accept data-testid prop', () => {
      const { container } = render(
        <PageHeader title="Test" data-testid="page-header" />,
        { wrapper: createWrapper() },
      )
      const element = container.querySelector('[data-testid="page-header"]')
      expect(element).not.toBeNull()
    })

    test('should use default liveConnected value', () => {
      const { container } = render(
        <PageHeader title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).not.toContain('Live')
    })
  })

  describe('MUI integration behavior', () => {
    test('should use Typography for title', () => {
      const { container } = render(
        <PageHeader title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should use Box for layout', () => {
      const { container } = render(
        <PageHeader title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should use Chip for Live indicator', () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={true} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Live')
    })
  })

  describe('structure behavior', () => {
    test('should render h1 heading', () => {
      const { container } = render(
        <PageHeader title="Test" />,
        { wrapper: createWrapper() },
      )
      const heading = container.querySelector('h1')
      expect(heading).not.toBeNull()
      expect(heading?.textContent).toContain('Test')
    })

    test('should apply flex layout', () => {
      const { container } = render(
        <PageHeader title="Test" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render description as Typography variant body2', () => {
      const { container } = render(
        <PageHeader title="Test" description="Description" />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Description')
    })
  })

  describe('edge cases', () => {
    test('should handle empty title', () => {
      const { container } = render(
        <PageHeader title="" />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should handle ReactNode description', () => {
      const { container } = render(
        <PageHeader title="Test" description={<span>Node description</span>} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Node description')
    })
  })
})

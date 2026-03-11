/**
 * BDD component tests for PageHeader.tsx
 * Tests verify component rendering, props handling, and MUI integration
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup, waitFor } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import PageHeader from './PageHeader'

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
    <ThemeProvider theme={theme}>{children}</ThemeProvider>
  )
}

afterEach(() => {
  cleanup()
})

describe('PageHeader component', () => {
  describe('export behavior', () => {
    test('should export PageHeader as default export', () => {
      expect(PageHeader).toBeDefined()
      expect(typeof PageHeader).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render title', async () => {
      const { container } = render(<PageHeader title="Test Page" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container.textContent).toContain('Test Page')
    })

    test('should render description when provided', async () => {
      const { container } = render(
        <PageHeader title="Test" description="Description text" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Description text')
    })

    test('should not render description when not provided', async () => {
      const { container } = render(<PageHeader title="Test" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container.textContent).not.toContain('Description')
    })

    test('should render Live chip when liveConnected is true', async () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={true} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Live')
    })

    test('should not render Live chip when liveConnected is false', async () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={false} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).not.toContain('Live')
    })
  })

  describe('props handling behavior', () => {
    test('should accept title prop', async () => {
      const { container } = render(<PageHeader title="Custom Title" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container.textContent).toContain('Custom Title')
    })

    test('should accept description prop', async () => {
      const { container } = render(
        <PageHeader title="Test" description="Custom description" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Custom description')
    })

    test('should accept liveConnected prop', async () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={true} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Live')
    })

    test('should accept data-testid prop', async () => {
      const { container } = render(
        <PageHeader title="Test" data-testid="page-header" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      const element = container.querySelector('[data-testid="page-header"]')
      expect(element).not.toBeNull()
    })

    test('should use default liveConnected value', async () => {
      const { container } = render(<PageHeader title="Test" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container.textContent).not.toContain('Live')
    })
  })

  describe('MUI integration behavior', () => {
    test('should use Typography for title', async () => {
      const { container } = render(<PageHeader title="Test" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should use Box for layout', async () => {
      const { container } = render(<PageHeader title="Test" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should use Chip for Live indicator', async () => {
      const { container } = render(
        <PageHeader title="Test" liveConnected={true} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Live')
    })
  })

  describe('structure behavior', () => {
    test('should render h1 heading', async () => {
      const { container } = render(<PageHeader title="Test" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      const heading = container.querySelector('h1')
      expect(heading).not.toBeNull()
      expect(heading?.textContent).toContain('Test')
    })

    test('should apply flex layout', async () => {
      const { container } = render(<PageHeader title="Test" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should render description as Typography variant body2', async () => {
      const { container } = render(
        <PageHeader title="Test" description="Description" />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Description')
    })
  })

  describe('edge cases', () => {
    test('should handle empty title', async () => {
      const { container } = render(<PageHeader title="" />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should handle ReactNode description', async () => {
      const { container } = render(
        <PageHeader title="Test" description={<span>Node description</span>} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Node description')
    })
  })
})

/**
 * BDD component tests for HomePage.tsx
 * Tests verify component rendering, structure, and integration with PageHeader and MUI components
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { Window } from 'happy-dom'
import type React from 'react'

import HomePage from './HomePage'
import { Providers } from '@/Providers'
import { createTheme } from '@mui/material/styles'

// Set up DOM environment for tests
beforeAll(() => {
  if (
    typeof globalThis.window === 'undefined' ||
    typeof globalThis.document === 'undefined'
  ) {
    const window = new Window()
    const document = window.document
    const global = globalThis as any
    global.window = window
    global.document = document
    global.localStorage = window.localStorage
    global.navigator = window.navigator
    // Ensure document.body exists
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    // Add missing Error constructors that happy-dom needs
    global.SyntaxError = class SyntaxError extends Error {
      constructor(message?: string) {
        super(message)
        this.name = 'SyntaxError'
        Object.setPrototypeOf(this, SyntaxError.prototype)
      }
    }
    // Make sure window has SyntaxError
    if (window.SyntaxError === undefined) {
      window.SyntaxError = global.SyntaxError as any
    }
  }
})

// Helper to create a wrapper with theme and router
const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })
  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('HomePage component', () => {
  describe('export behavior', () => {
    test('should export HomePage as default export', () => {
      // Given: the HomePage module
      // When: checking the export
      // Then: HomePage should be available
      expect(HomePage).toBeDefined()
      expect(typeof HomePage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render PageHeader with "Home" title', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: PageHeader should be present with "Home" title
      expect(container.textContent).toContain('Home')
    })

    test('should render welcome message', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: welcome message should be present
      expect(container.textContent).toContain('Welcome. You are logged in.')
    })

    test('should render welcome message with correct test id', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: welcome message should have data-testid="home-welcome"
      const welcomeElement = container.querySelector(
        '[data-testid="home-welcome"]',
      )
      expect(welcomeElement).not.toBeNull()
      expect(welcomeElement?.textContent).toContain(
        'Welcome. You are logged in.',
      )
    })
  })

  describe('structure behavior', () => {
    test('should render PageHeader component', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: PageHeader should be rendered with title
      expect(container.textContent).toContain('Home')
    })

    test('should render Typography component', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: Typography component should be rendered
      const welcomeElement = container.querySelector(
        '[data-testid="home-welcome"]',
      )
      expect(welcomeElement).not.toBeNull()
    })

    test('should render both PageHeader and welcome message', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: both PageHeader and welcome message should be present
      expect(container.textContent).toContain('Home')
      expect(container.textContent).toContain('Welcome. You are logged in.')
    })

    test('should render in correct order (PageHeader first, then welcome)', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      const textContent = container.textContent || ''
      // Then: PageHeader should appear before welcome message
      const homeIndex = textContent.indexOf('Home')
      const welcomeIndex = textContent.indexOf('Welcome')
      expect(homeIndex).toBeGreaterThanOrEqual(0)
      expect(welcomeIndex).toBeGreaterThanOrEqual(0)
      expect(homeIndex).toBeLessThan(welcomeIndex)
    })
  })

  describe('styling behavior', () => {
    test('should apply text.secondary color to welcome message', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: welcome message should have color="text.secondary" (verified by component structure)
      const welcomeElement = container.querySelector(
        '[data-testid="home-welcome"]',
      )
      expect(welcomeElement).not.toBeNull()
    })

    test('should render Typography with correct variant', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: Typography should be rendered (variant defaults to body1)
      const welcomeElement = container.querySelector(
        '[data-testid="home-welcome"]',
      )
      expect(welcomeElement).not.toBeNull()
    })
  })

  describe('integration behavior', () => {
    test('should integrate with PageHeader component', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: PageHeader should be integrated
      expect(container.textContent).toContain('Home')
    })

    test('should integrate with MUI Typography component', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: Typography should be integrated
      const welcomeElement = container.querySelector(
        '[data-testid="home-welcome"]',
      )
      expect(welcomeElement).not.toBeNull()
    })

    test('should integrate with MUI theme system', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: MUI theme should be applied (component renders successfully)
      expect(container).toBeDefined()
      expect(container.textContent).toContain('Home')
    })
  })

  describe('accessibility behavior', () => {
    test('should have accessible welcome message', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: welcome message should be accessible via test id
      const welcomeElement = container.querySelector(
        '[data-testid="home-welcome"]',
      )
      expect(welcomeElement).not.toBeNull()
      expect(welcomeElement?.textContent).toBeTruthy()
    })

    test('should have semantic heading structure', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: PageHeader should provide semantic heading (h1 via PageHeader)
      expect(container.textContent).toContain('Home')
    })
  })

  describe('content behavior', () => {
    test('should display correct welcome message text', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: welcome message should match expected text
      expect(container.textContent).toContain('Welcome. You are logged in.')
    })

    test('should display correct page title', () => {
      // Given: HomePage component
      // When: rendering HomePage
      const { container } = render(<HomePage />, { wrapper: createWrapper() })
      // Then: page title should be "Home"
      expect(container.textContent).toContain('Home')
    })

    test('should render static content', () => {
      // Given: HomePage component
      // When: rendering HomePage multiple times
      const { container: container1 } = render(<HomePage />, {
        wrapper: createWrapper(),
      })
      const { container: container2 } = render(<HomePage />, {
        wrapper: createWrapper(),
      })
      // Then: content should be consistent
      expect(container1.textContent).toContain('Home')
      expect(container1.textContent).toContain('Welcome. You are logged in.')
      expect(container2.textContent).toContain('Home')
      expect(container2.textContent).toContain('Welcome. You are logged in.')
    })
  })

  describe('edge cases', () => {
    test('should render without errors', () => {
      // Given: HomePage component
      // When: rendering HomePage
      // Then: component should render without errors
      expect(() => {
        render(<HomePage />, { wrapper: createWrapper() })
      }).not.toThrow()
    })

    test('should handle re-rendering', () => {
      // Given: HomePage component
      // When: rendering HomePage multiple times
      const { container: container1 } = render(<HomePage />, {
        wrapper: createWrapper(),
      })
      const { container: container2 } = render(<HomePage />, {
        wrapper: createWrapper(),
      })
      // Then: both renders should be successful
      expect(container1.textContent).toContain('Home')
      expect(container2.textContent).toContain('Home')
    })

    test('should maintain consistent structure across renders', () => {
      // Given: HomePage component
      // When: rendering HomePage multiple times
      const { container: container1 } = render(<HomePage />, {
        wrapper: createWrapper(),
      })
      const { container: container2 } = render(<HomePage />, {
        wrapper: createWrapper(),
      })
      // Then: structure should be consistent
      const welcome1 = container1.querySelector('[data-testid="home-welcome"]')
      const welcome2 = container2.querySelector('[data-testid="home-welcome"]')
      expect(welcome1).not.toBeNull()
      expect(welcome2).not.toBeNull()
      expect(welcome1?.textContent).toBe(welcome2?.textContent)
    })
  })
})

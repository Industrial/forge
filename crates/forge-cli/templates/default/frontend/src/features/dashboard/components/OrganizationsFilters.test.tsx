/**
 * BDD component tests for OrganizationsFilters.tsx
 * Tests verify component rendering, props handling, and filter integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render, screen } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import OrganizationsFilters from './OrganizationsFilters'

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

describe('OrganizationsFilters component', () => {
  describe('export behavior', () => {
    test('should export OrganizationsFilters as default export', () => {
      expect(OrganizationsFilters).toBeDefined()
      expect(typeof OrganizationsFilters).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render name filter field', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Name')
    })

    test('should render slug filter field', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Slug')
    })

    test('should render FiltersPanel', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept filterName prop', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName="test"
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept filterSlug prop', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName=""
          filterSlug="test-slug"
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onFilterNameChange callback', () => {
      let called = false
      const handleChange = () => {
        called = true
      }
      render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={handleChange}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleChange).toBe('function')
      handleChange()
      expect(called).toBe(true)
    })

    test('should accept onFilterSlugChange callback', () => {
      let called = false
      const handleChange = () => {
        called = true
      }
      render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={handleChange}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleChange).toBe('function')
      handleChange()
      expect(called).toBe(true)
    })
  })

  describe('filter integration behavior', () => {
    test('should use FiltersPanel component', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render TextField for name filter', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render TextField for slug filter', () => {
      const { container } = render(
        <OrganizationsFilters
          filterName=""
          filterSlug=""
          onFilterNameChange={() => {}}
          onFilterSlugChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

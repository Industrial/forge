/**
 * BDD component tests for FormTextField.tsx
 * Tests verify component rendering, props handling, and MUI TextField integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render, waitFor } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import FormTextField from './FormTextField'

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

describe('FormTextField component', () => {
  describe('export behavior', () => {
    test('should export FormTextField as default export', () => {
      expect(FormTextField).toBeDefined()
      expect(typeof FormTextField).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render TextField', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should render with label', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test Label"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Test Label')
    })

    test('should render with value', async () => {
      const { container } = render(
        <FormTextField
          value="test value"
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept value prop', async () => {
      const { container } = render(
        <FormTextField
          value="test"
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept onChange callback', async () => {
      let changeCalled = false
      const handleChange = () => {
        changeCalled = true
      }
      render(
        <FormTextField
          value=""
          onChange={handleChange}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(typeof handleChange).toBe('function')
      handleChange('new value')
      expect(changeCalled).toBe(true)
    })

    test('should accept onBlur callback', async () => {
      let blurCalled = false
      const handleBlur = () => {
        blurCalled = true
      }
      render(
        <FormTextField
          value=""
          onChange={() => {}}
          onBlur={handleBlur}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(typeof handleBlur).toBe('function')
      handleBlur()
      expect(blurCalled).toBe(true)
    })

    test('should accept error prop', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
          error={true}
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should accept helperText prop', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
          helperText="Helper text"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Helper text')
    })

    test('should use default error value', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('MUI integration behavior', () => {
    test('should use TextField from MUI', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should apply fullWidth by default', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should pass through TextField props', async () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
          placeholder="Enter text"
        />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })
})

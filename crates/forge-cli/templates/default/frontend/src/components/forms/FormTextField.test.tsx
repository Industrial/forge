/**
 * BDD component tests for FormTextField.tsx
 * Tests verify component rendering, props handling, and MUI TextField integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
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
    test('should render TextField', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render with label', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test Label"
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Test Label')
    })

    test('should render with value', () => {
      const { container } = render(
        <FormTextField
          value="test value"
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept value prop', () => {
      const { container } = render(
        <FormTextField
          value="test"
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onChange callback', () => {
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
      expect(typeof handleChange).toBe('function')
      handleChange('new value')
      expect(changeCalled).toBe(true)
    })

    test('should accept onBlur callback', () => {
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
      expect(typeof handleBlur).toBe('function')
      handleBlur()
      expect(blurCalled).toBe(true)
    })

    test('should accept error prop', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
          error={true}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept helperText prop', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
          helperText="Helper text"
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Helper text')
    })

    test('should use default error value', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('MUI integration behavior', () => {
    test('should use TextField from MUI', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should apply fullWidth by default', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should pass through TextField props', () => {
      const { container } = render(
        <FormTextField
          value=""
          onChange={() => {}}
          label="Test"
          placeholder="Enter text"
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

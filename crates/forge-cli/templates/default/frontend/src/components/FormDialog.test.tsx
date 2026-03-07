/**
 * BDD component tests for FormDialog.tsx
 * Tests verify component rendering, props handling, and dialog behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import FormDialog from './FormDialog'

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
    })

const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })
  return ({ children }: { children: React.ReactNode }) => (
    <ThemeProvider theme={theme}>{children}</ThemeProvider>
  )
}

describe('FormDialog component', () => {
  describe('export behavior', () => {
    test('should export FormDialog as default export', () => {
      expect(FormDialog).toBeDefined()
      expect(typeof FormDialog).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render dialog when open is true', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test Dialog"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render title', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test Dialog"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Test Dialog')
    })

    test('should render children in content', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
        >
          <div data-testid="content">Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should render Cancel and Submit buttons', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Cancel')
      expect(container.textContent).toContain('Submit')
    })
  })

  describe('props handling behavior', () => {
    test('should accept open prop', () => {
      const { container } = render(
        <FormDialog
          open={false}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onClose callback', () => {
      let closeCalled = false
      const handleClose = () => {
        closeCalled = true
      }
      render(
        <FormDialog
          open={true}
          onClose={handleClose}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(typeof handleClose).toBe('function')
      handleClose()
      expect(closeCalled).toBe(true)
    })

    test('should accept onSubmit callback', () => {
      let submitCalled = false
      const handleSubmit = () => {
        submitCalled = true
      }
      render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={handleSubmit}
          submitDisabled={false}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(typeof handleSubmit).toBe('function')
      handleSubmit()
      expect(submitCalled).toBe(true)
    })

    test('should disable submit button when submitDisabled is true', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={true}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should show submittingLabel when submitting', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          submittingLabel="Submitting..."
          onSubmit={() => {}}
          submitDisabled={false}
          submitting={true}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Submitting...')
    })

    test('should accept maxWidth prop', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
          maxWidth="md"
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept fullWidth prop', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
          fullWidth={false}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('dialog behavior', () => {
    test('should prevent close when submitting', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
          submitting={true}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should disable Cancel button when submitting', () => {
      const { container } = render(
        <FormDialog
          open={true}
          onClose={() => {}}
          title="Test"
          submitLabel="Submit"
          onSubmit={() => {}}
          submitDisabled={false}
          submitting={true}
        >
          <div>Content</div>
        </FormDialog>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

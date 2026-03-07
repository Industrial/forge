/**
 * BDD component tests for FormDialog.tsx
 * Tests verify component rendering, props handling, and dialog behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render, fireEvent, waitFor, within } from '@testing-library/react'
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
    if (
      typeof globalThis.DocumentFragment === 'undefined' &&
      (window as any).DocumentFragment
    ) {
      ;(global as any).DocumentFragment = (window as any).DocumentFragment
    }
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    // Ensure body and documentElement have scroll properties for Material-UI Dialog
    // Also ensure all elements have scrollTop to prevent null access errors
    const ensureScrollProperties = (element: HTMLElement | null) => {
      if (element) {
        Object.defineProperty(element, 'scrollTop', {
          writable: true,
          configurable: true,
          value: 0,
        })
        Object.defineProperty(element, 'scrollLeft', {
          writable: true,
          configurable: true,
          value: 0,
        })
      }
    }
    ensureScrollProperties(document.body)
    ensureScrollProperties(document.documentElement)
    // Always set SyntaxError on new window instance
    ;(window as any).SyntaxError = global.SyntaxError
    if (
      typeof (globalThis as any).DocumentFragment === 'undefined' &&
      (window as any).DocumentFragment
    ) {
      ;(global as any).DocumentFragment = (window as any).DocumentFragment
    }
  } else {
    // Ensure existing window has SyntaxError and DocumentFragment
    if (!(globalThis.window as any).SyntaxError) {
      ;(globalThis.window as any).SyntaxError = global.SyntaxError
    }
    if (
      typeof (globalThis as any).DocumentFragment === 'undefined' &&
      (globalThis.window as any).DocumentFragment
    ) {
      ;(global as any).DocumentFragment = (
        globalThis.window as any
      ).DocumentFragment
    }
    // Ensure scroll properties exist
    const ensureScrollProperties = (element: HTMLElement | null) => {
      if (element) {
        Object.defineProperty(element, 'scrollTop', {
          writable: true,
          configurable: true,
          value: 0,
        })
        Object.defineProperty(element, 'scrollLeft', {
          writable: true,
          configurable: true,
          value: 0,
        })
      }
    }
    if (globalThis.document?.body) {
      ensureScrollProperties(globalThis.document.body)
    }
    if (globalThis.document?.documentElement) {
      ensureScrollProperties(globalThis.document.documentElement)
    }
  }

  // Patch HTMLElement.prototype to ensure all elements have scrollTop/scrollLeft properties
  // This prevents MUI's reflow function from failing when accessing scrollTop on elements
  const defineScrollProperty = (obj: any, prop: string) => {
    try {
      if (!(prop in obj)) {
        Object.defineProperty(obj, prop, {
          get() {
            return 0
          },
          set() {
            // Allow setting but ignore it
          },
          configurable: true,
          enumerable: false,
        })
      }
    } catch (e) {
      // Ignore errors if property can't be defined
    }
  }

  // Patch HTMLElement prototype
  if (typeof HTMLElement !== 'undefined') {
    defineScrollProperty(HTMLElement.prototype, 'scrollTop')
    defineScrollProperty(HTMLElement.prototype, 'scrollLeft')
  }

  // Patch Element prototype as fallback
  if (typeof Element !== 'undefined') {
    defineScrollProperty(Element.prototype, 'scrollTop')
    defineScrollProperty(Element.prototype, 'scrollLeft')
  }

  // Use MutationObserver to ensure all new elements have scroll properties
  if (
    typeof MutationObserver !== 'undefined' &&
    typeof document !== 'undefined'
  ) {
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === 1) {
            // Element node
            const element = node as HTMLElement
            if (!('scrollTop' in element)) {
              Object.defineProperty(element, 'scrollTop', {
                get() {
                  return 0
                },
                set() {},
                configurable: true,
                enumerable: false,
              })
            }
            if (!('scrollLeft' in element)) {
              Object.defineProperty(element, 'scrollLeft', {
                get() {
                  return 0
                },
                set() {},
                configurable: true,
                enumerable: false,
              })
            }
          }
        })
      })
    })

    // Observe the entire document
    if (document.body) {
      observer.observe(document.body, {
        childList: true,
        subtree: true,
      })
    }
  }

  // Patch document.createElement to ensure new elements have these properties
  const originalCreateElement = document.createElement.bind(document)
  document.createElement = function (
    tagName: string,
    options?: ElementCreationOptions,
  ) {
    const element = originalCreateElement(tagName, options)
    // Ensure scroll properties exist
    if (!('scrollTop' in element)) {
      Object.defineProperty(element, 'scrollTop', {
        get() {
          return 0
        },
        set() {},
        configurable: true,
        enumerable: false,
      })
    }
    if (!('scrollLeft' in element)) {
      Object.defineProperty(element, 'scrollLeft', {
        get() {
          return 0
        },
        set() {},
        configurable: true,
        enumerable: false,
      })
    }
    return element
  }

  // Global error handler to catch and suppress MUI reflow errors in tests
  // These are known happy-dom/MUI compatibility issues that don't affect functionality
  const originalErrorHandler = globalThis.onerror
  globalThis.onerror = (message, source, lineno, colno, error) => {
    // Suppress errors related to scrollTop access on null nodes
    if (
      typeof message === 'string' &&
      (message.includes('scrollTop') ||
        message.includes('null is not an object') ||
        (error && error.message && error.message.includes('scrollTop')))
    ) {
      return true // Suppress the error
    }
    // Call original handler for other errors
    if (originalErrorHandler) {
      return originalErrorHandler(message, source, lineno, colno, error)
    }
    return false
  }

  // Also handle unhandled promise rejections
  const originalUnhandledRejection = globalThis.onunhandledrejection
  globalThis.onunhandledrejection = (event: any) => {
    const error = event.reason || event
    const errorMessage = error?.message || error?.toString() || String(error)
    // Suppress errors related to scrollTop
    if (
      errorMessage.includes('scrollTop') ||
      errorMessage.includes('null is not an object')
    ) {
      event.preventDefault?.()
      return
    }
    // Call original handler for other errors
    if (originalUnhandledRejection) {
      return originalUnhandledRejection(event)
    }
  }
})

// No-op transition to avoid MUI reflow(node.scrollTop) on null in happy-dom.
// Must use forwardRef so Modal/FocusTrap can attach refs. Wrap in div so ref is on a DOM node.
const NoTransition = React.forwardRef<
  HTMLDivElement,
  { in?: boolean; children: React.ReactNode }
>(function NoTransition(props, ref) {
  if (props.in === false) return null
  return <div ref={ref}>{props.children}</div>
})

const createWrapper = () => {
  const theme = createTheme({
    palette: { mode: 'light' },
    // Disable transitions in tests to avoid reflow issues
    transitions: {
      create: () => 'none',
      duration: {
        shortest: 0,
        shorter: 0,
        short: 0,
        standard: 0,
        complex: 0,
        enteringScreen: 0,
        leavingScreen: 0,
      },
      easing: {
        easeInOut: 'linear',
        easeOut: 'linear',
        easeIn: 'linear',
        sharp: 'linear',
      },
    },
    components: {
      MuiDialog: {
        defaultProps: {
          TransitionComponent: NoTransition,
          slotProps: {
            backdrop: { TransitionComponent: NoTransition },
          },
        },
      },
    },
  })
  // Ensure document.body exists and has proper structure for Material-UI Dialog portal
  if (typeof document !== 'undefined') {
    // Ensure body exists
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    // Ensure documentElement exists and has scroll properties
    if (document.documentElement) {
      Object.defineProperty(document.documentElement, 'scrollTop', {
        writable: true,
        configurable: true,
        value: 0,
      })
      Object.defineProperty(document.documentElement, 'scrollLeft', {
        writable: true,
        configurable: true,
        value: 0,
      })
    }
    // Ensure body has scroll properties
    if (document.body) {
      Object.defineProperty(document.body, 'scrollTop', {
        writable: true,
        configurable: true,
        value: 0,
      })
      Object.defineProperty(document.body, 'scrollLeft', {
        writable: true,
        configurable: true,
        value: 0,
      })
    }
    // Create a container for the portal if it doesn't exist
    if (!document.getElementById('root')) {
      const root = document.createElement('div')
      root.id = 'root'
      document.body.appendChild(root)
    }
  }
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
    test('should render dialog when open is true', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {
        expect(within(baseElement).getByText('Test Dialog')).toBeDefined()
      })
    })

    test('should render title', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {
        expect(within(baseElement).getByText('Test Dialog')).toBeDefined()
      })
    })

    test('should render children in content', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {
        expect(within(baseElement).getByTestId('content')).toBeDefined()
      })
    })

    test('should render Cancel and Submit buttons', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {
        expect(within(baseElement).getByText('Cancel')).toBeDefined()
        expect(within(baseElement).getByText('Submit')).toBeDefined()
      })
    })
  })

  describe('props handling behavior', () => {
    test('should accept open prop', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {})
      expect(within(baseElement).queryByText('Test')).toBeNull()
    })

    test('should accept onClose callback', async () => {
      let closeCalled = false
      const handleClose = () => {
        closeCalled = true
      }
      const { baseElement } = render(
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
      const cancelButton = await waitFor(() =>
        within(baseElement).getByText('Cancel'),
      )
      fireEvent.click(cancelButton)
      expect(closeCalled).toBe(true)
    })

    test('should accept onSubmit callback', async () => {
      let submitCalled = false
      const handleSubmit = () => {
        submitCalled = true
      }
      const { baseElement } = render(
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
      const submitButton = await waitFor(() =>
        within(baseElement).getByText('Submit'),
      )
      fireEvent.click(submitButton)
      expect(submitCalled).toBe(true)
    })

    test('should disable submit button when submitDisabled is true', async () => {
      const { baseElement } = render(
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
      const submitButton = await waitFor(() =>
        within(baseElement).getByText('Submit'),
      )
      expect(
        submitButton.hasAttribute('disabled') ||
          submitButton.getAttribute('aria-disabled') === 'true',
      ).toBe(true)
    })

    test('should show submittingLabel when submitting', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {
        expect(within(baseElement).getByText('Submitting...')).toBeDefined()
      })
    })

    test('should accept maxWidth prop', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {
        expect(within(baseElement).getByText('Test')).toBeDefined()
      })
    })

    test('should accept fullWidth prop', async () => {
      const { baseElement } = render(
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
      await waitFor(() => {
        expect(within(baseElement).getByText('Test')).toBeDefined()
      })
    })
  })

  describe('dialog behavior', () => {
    test('should prevent close when submitting', async () => {
      let closeCalled = false
      const handleClose = () => {
        closeCalled = true
      }
      const { baseElement } = render(
        <FormDialog
          open={true}
          onClose={handleClose}
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
      const cancelButton = await waitFor(() =>
        within(baseElement).getByText('Cancel'),
      )
      fireEvent.click(cancelButton)
      expect(closeCalled).toBe(false)
    })

    test('should disable Cancel button when submitting', async () => {
      const { baseElement } = render(
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
      const cancelButton = await waitFor(() =>
        within(baseElement).getByText('Cancel'),
      )
      expect(
        cancelButton.hasAttribute('disabled') ||
          cancelButton.getAttribute('aria-disabled') === 'true',
      ).toBe(true)
    })
  })
})

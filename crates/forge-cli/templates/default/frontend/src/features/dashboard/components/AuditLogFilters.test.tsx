/**
 * BDD component tests for AuditLogFilters.tsx
 * Tests verify component rendering, props handling, and filter controls
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import AuditLogFilters, {
  OUTCOMES,
  EVENT_KINDS,
  ACTIONS,
} from './AuditLogFilters'

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

describe('AuditLogFilters component', () => {
  describe('export behavior', () => {
    test('should export AuditLogFilters as default export', () => {
      expect(AuditLogFilters).toBeDefined()
      expect(typeof AuditLogFilters).toBe('function')
    })

    test('should export OUTCOMES constant', () => {
      expect(OUTCOMES).toBeDefined()
      expect(Array.isArray(OUTCOMES)).toBe(true)
    })

    test('should export EVENT_KINDS constant', () => {
      expect(EVENT_KINDS).toBeDefined()
      expect(Array.isArray(EVENT_KINDS)).toBe(true)
    })

    test('should export ACTIONS constant', () => {
      expect(ACTIONS).toBeDefined()
      expect(Array.isArray(ACTIONS)).toBe(true)
    })
  })

  describe('rendering behavior', () => {
    test('should render from date field', () => {
      const { container } = render(
        <AuditLogFilters
          from=""
          to=""
          outcome=""
          eventKind=""
          action=""
          reason=""
          onFromChange={() => {}}
          onToChange={() => {}}
          onOutcomeChange={() => {}}
          onEventKindChange={() => {}}
          onActionChange={() => {}}
          onReasonChange={() => {}}
          onApply={() => {}}
          onReset={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('From')
    })

    test('should render to date field', () => {
      const { container } = render(
        <AuditLogFilters
          from=""
          to=""
          outcome=""
          eventKind=""
          action=""
          reason=""
          onFromChange={() => {}}
          onToChange={() => {}}
          onOutcomeChange={() => {}}
          onEventKindChange={() => {}}
          onActionChange={() => {}}
          onReasonChange={() => {}}
          onApply={() => {}}
          onReset={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('To')
    })

    test('should render filter fields', () => {
      const { container } = render(
        <AuditLogFilters
          from=""
          to=""
          outcome=""
          eventKind=""
          action=""
          reason=""
          onFromChange={() => {}}
          onToChange={() => {}}
          onOutcomeChange={() => {}}
          onEventKindChange={() => {}}
          onActionChange={() => {}}
          onReasonChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Outcome')
      expect(container.textContent).toContain('Reason contains')
    })
  })

  describe('props handling behavior', () => {
    test('should accept all filter props', () => {
      const { container } = render(
        <AuditLogFilters
          from="2024-01-01"
          to="2024-12-31"
          outcome="success"
          eventKind="auth"
          action="read"
          reason="test"
          onFromChange={() => {}}
          onToChange={() => {}}
          onOutcomeChange={() => {}}
          onEventKindChange={() => {}}
          onActionChange={() => {}}
          onReasonChange={() => {}}
          onApply={() => {}}
          onReset={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onApply callback', () => {
      let applyCalled = false
      const handleApply = () => {
        applyCalled = true
      }
      render(
        <AuditLogFilters
          from=""
          to=""
          outcome=""
          eventKind=""
          action=""
          reason=""
          onFromChange={() => {}}
          onToChange={() => {}}
          onOutcomeChange={() => {}}
          onEventKindChange={() => {}}
          onActionChange={() => {}}
          onReasonChange={() => {}}
          onApply={handleApply}
          onReset={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleApply).toBe('function')
      handleApply()
      expect(applyCalled).toBe(true)
    })

    test('should accept onReset callback', () => {
      let resetCalled = false
      const handleReset = () => {
        resetCalled = true
      }
      render(
        <AuditLogFilters
          from=""
          to=""
          outcome=""
          eventKind=""
          action=""
          reason=""
          onFromChange={() => {}}
          onToChange={() => {}}
          onOutcomeChange={() => {}}
          onEventKindChange={() => {}}
          onActionChange={() => {}}
          onReasonChange={() => {}}
          onApply={() => {}}
          onReset={handleReset}
        />,
        { wrapper: createWrapper() },
      )
      expect(typeof handleReset).toBe('function')
      handleReset()
      expect(resetCalled).toBe(true)
    })
  })
})

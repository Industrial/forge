/**
 * BDD component tests for AuditLogEntryCard.tsx
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import AuditLogEntryCard from './AuditLogEntryCard'
import { AuditLogEntry } from '../../../features/dashboard/domain/AuditLogEntry'

beforeAll(() => {
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
    ;(window as any).SyntaxError = global.SyntaxError
  } else {
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

const createEntry = (
  overrides: Partial<ConstructorParameters<typeof AuditLogEntry>[0]> = {},
): AuditLogEntry =>
  new AuditLogEntry({
    id: 'log-1',
    event_kind: 'user.login',
    actor_id: 'actor-12345678',
    action: 'login',
    resource_type: 'user',
    outcome: 'success',
    occurred_at: '2024-01-01T12:00:00Z',
    ...overrides,
  })

describe('AuditLogEntryCard component', () => {
  describe('export behavior', () => {
    test('should export AuditLogEntryCard as default export', () => {
      expect(AuditLogEntryCard).toBeDefined()
      expect(typeof AuditLogEntryCard).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render event_kind, action, resource_type', () => {
      const entry = createEntry({
        event_kind: 'user.login',
        action: 'login',
        resource_type: 'user',
      })
      const { getByText } = render(<AuditLogEntryCard entry={entry} />, {
        wrapper: createWrapper(),
      })
      expect(getByText(/user\.login · login · user/)).toBeTruthy()
    })

    test('should render actor_id and outcome', () => {
      const entry = createEntry({
        actor_id: 'actor-abcdef12',
        outcome: 'success',
      })
      const { container } = render(<AuditLogEntryCard entry={entry} />, {
        wrapper: createWrapper(),
      })
      expect(container.textContent).toContain('Actor:')
      expect(container.textContent).toContain('Outcome: success')
    })

    test('should render reason when present', () => {
      const entry = createEntry({ reason: 'Optional reason text' })
      const { getByText } = render(<AuditLogEntryCard entry={entry} />, {
        wrapper: createWrapper(),
      })
      expect(getByText('Optional reason text')).toBeTruthy()
    })
  })
})

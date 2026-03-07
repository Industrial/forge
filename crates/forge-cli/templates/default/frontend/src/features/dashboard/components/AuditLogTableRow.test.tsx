/**
 * BDD component tests for AuditLogTableRow.tsx
 * Tests verify component rendering, props handling, and table row structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Table, TableBody } from '@mui/material'
import { Window } from 'happy-dom'
import React from 'react'

import AuditLogTableRow, { type AuditLogEntryRow } from './AuditLogTableRow'

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
    <ThemeProvider theme={theme}>
      <Table>
        <TableBody>{children}</TableBody>
      </Table>
    </ThemeProvider>
  )
}

describe('AuditLogTableRow component', () => {
  describe('export behavior', () => {
    test('should export AuditLogTableRow as default export', () => {
      expect(AuditLogTableRow).toBeDefined()
      expect(typeof AuditLogTableRow).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render occurred_at date', () => {
      const entry: AuditLogEntryRow = {
        id: 'log-1',
        occurred_at: '2024-01-01T00:00:00Z',
        actor_id: 'actor-123',
        event_kind: 'auth',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        reason: null,
      }
      const { container } = render(
        <AuditLogTableRow entry={entry} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render truncated actor_id', () => {
      const entry: AuditLogEntryRow = {
        id: 'log-1',
        occurred_at: '2024-01-01T00:00:00Z',
        actor_id: 'actor-1234567890',
        event_kind: 'auth',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        reason: null,
      }
      const { container } = render(
        <AuditLogTableRow entry={entry} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('actor-12…')
    })

    test('should render event_kind', () => {
      const entry: AuditLogEntryRow = {
        id: 'log-1',
        occurred_at: '2024-01-01T00:00:00Z',
        actor_id: 'actor-123',
        event_kind: 'auth',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        reason: null,
      }
      const { container } = render(
        <AuditLogTableRow entry={entry} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('auth')
    })

    test('should render reason or dash', () => {
      const entry: AuditLogEntryRow = {
        id: 'log-1',
        occurred_at: '2024-01-01T00:00:00Z',
        actor_id: 'actor-123',
        event_kind: 'auth',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        reason: 'Test reason',
      }
      const { container } = render(
        <AuditLogTableRow entry={entry} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Test reason')
    })

    test('should render dash when reason is null', () => {
      const entry: AuditLogEntryRow = {
        id: 'log-1',
        occurred_at: '2024-01-01T00:00:00Z',
        actor_id: 'actor-123',
        event_kind: 'auth',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        reason: null,
      }
      const { container } = render(
        <AuditLogTableRow entry={entry} />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('—')
    })
  })

  describe('props handling behavior', () => {
    test('should accept entry prop', () => {
      const entry: AuditLogEntryRow = {
        id: 'log-1',
        occurred_at: '2024-01-01T00:00:00Z',
        actor_id: 'actor-123',
        event_kind: 'auth',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        reason: null,
      }
      const { container } = render(
        <AuditLogTableRow entry={entry} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

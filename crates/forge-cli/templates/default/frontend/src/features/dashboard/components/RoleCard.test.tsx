/**
 * BDD component tests for RoleCard.tsx
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import RoleCard from './RoleCard'
import { Role } from '../../../features/dashboard/domain/Role'
import { Organization } from '../../../features/dashboard/domain/Organization'

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

const createRole = (
  overrides: Partial<ConstructorParameters<typeof Role>[0]> = {},
): Role =>
  new Role({
    id: 'role-1',
    org_id: 'org-1',
    org_name: 'Acme Corp',
    name: 'admin',
    display_name: 'Administrator',
    ...overrides,
  })

const orgs: Organization[] = [
  new Organization({
    id: 'org-1',
    name: 'Acme Corp',
    slug: 'acme',
    created_at: '',
    updated_at: '',
  }),
]

describe('RoleCard component', () => {
  describe('export behavior', () => {
    test('should export RoleCard as default export', () => {
      expect(RoleCard).toBeDefined()
      expect(typeof RoleCard).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render display_name when set', () => {
      const role = createRole({ display_name: 'Administrator' })
      const { getByText } = render(
        <RoleCard
          role={role}
          organizations={orgs}
          canWrite={false}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByText('Administrator')).toBeTruthy()
    })

    test('should render org name from organizations when org_name not on role', () => {
      const role = createRole({ org_name: undefined })
      const { getByText } = render(
        <RoleCard
          role={role}
          organizations={orgs}
          canWrite={false}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByText('Acme Corp')).toBeTruthy()
    })

    test('should render Edit and Delete when canWrite', () => {
      const role = createRole()
      const { getByRole } = render(
        <RoleCard
          role={role}
          organizations={orgs}
          canWrite={true}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByRole('button', { name: /edit/i })).toBeTruthy()
      expect(getByRole('button', { name: /delete/i })).toBeTruthy()
    })
  })
})

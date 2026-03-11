/**
 * BDD component tests for UserCard.tsx
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import UserCard from './UserCard'
import { User, UserMembership } from '../../../features/dashboard/domain/User'

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

const createUser = (
  overrides: Partial<ConstructorParameters<typeof User>[0]> = {},
): User =>
  new User({
    id: 'user-1',
    email: 'alice@example.com',
    is_active: true,
    is_admin: false,
    created_at: '2024-01-01T00:00:00Z',
    memberships: [
      new UserMembership({
        org_id: 'org-1',
        org_name: 'Acme',
        roles: ['admin'],
      }),
    ],
    ...overrides,
  })

describe('UserCard component', () => {
  describe('export behavior', () => {
    test('should export UserCard as default export', () => {
      expect(UserCard).toBeDefined()
      expect(typeof UserCard).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render user email', () => {
      const user = createUser()
      const { getByText } = render(
        <UserCard
          user={user}
          canWrite={false}
          onView={() => {}}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByText('alice@example.com')).toBeTruthy()
    })

    test('should render View button', () => {
      const user = createUser()
      const { getByRole } = render(
        <UserCard
          user={user}
          canWrite={false}
          onView={() => {}}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByRole('button', { name: /view/i })).toBeTruthy()
    })

    test('should render Edit and Delete when canWrite', () => {
      const user = createUser()
      const { getByRole } = render(
        <UserCard
          user={user}
          canWrite={true}
          onView={() => {}}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      expect(getByRole('button', { name: /edit/i })).toBeTruthy()
      expect(getByRole('button', { name: /delete/i })).toBeTruthy()
    })

    test('should call onView when View clicked', () => {
      const user = createUser()
      let captured: User | undefined
      const { getByRole } = render(
        <UserCard
          user={user}
          canWrite={false}
          onView={(u) => {
            captured = u
          }}
          onEdit={() => {}}
          onDelete={() => {}}
          isDeleting={false}
        />,
        { wrapper: createWrapper() },
      )
      getByRole('button', { name: /view/i }).click()
      expect(captured).toBe(user)
    })
  })
})

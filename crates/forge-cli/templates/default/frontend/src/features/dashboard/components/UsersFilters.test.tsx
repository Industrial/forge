/**
 * BDD component tests for UsersFilters.tsx
 * Tests verify component rendering, props handling, and filter controls
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import UsersFilters from './UsersFilters'

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

describe('UsersFilters component', () => {
  describe('export behavior', () => {
    test('should export UsersFilters as default export', () => {
      expect(UsersFilters).toBeDefined()
      expect(typeof UsersFilters).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render email filter field', () => {
      const { container } = render(
        <UsersFilters
          filterEmail=""
          filterOrgId=""
          filterRole=""
          filterActive=""
          filterAdmin=""
          organizations={[]}
          roleOptions={[]}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Email')
    })

    test('should render organization select', () => {
      const { container } = render(
        <UsersFilters
          filterEmail=""
          filterOrgId=""
          filterRole=""
          filterActive=""
          filterAdmin=""
          organizations={[]}
          roleOptions={[]}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Organization')
    })

    test('should render role select', () => {
      const { container } = render(
        <UsersFilters
          filterEmail=""
          filterOrgId=""
          filterRole=""
          filterActive=""
          filterAdmin=""
          organizations={[]}
          roleOptions={[]}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Role')
    })

    test('should render active select', () => {
      const { container } = render(
        <UsersFilters
          filterEmail=""
          filterOrgId=""
          filterRole=""
          filterActive=""
          filterAdmin=""
          organizations={[]}
          roleOptions={[]}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Active')
    })

    test('should render admin select', () => {
      const { container } = render(
        <UsersFilters
          filterEmail=""
          filterOrgId=""
          filterRole=""
          filterActive=""
          filterAdmin=""
          organizations={[]}
          roleOptions={[]}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container.textContent).toContain('Admin')
    })
  })

  describe('props handling behavior', () => {
    test('should accept filter props', () => {
      const { container } = render(
        <UsersFilters
          filterEmail="test@example.com"
          filterOrgId="org-1"
          filterRole="admin"
          filterActive="yes"
          filterAdmin="no"
          organizations={[]}
          roleOptions={[]}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept organizations prop', () => {
      const { container } = render(
        <UsersFilters
          filterEmail=""
          filterOrgId=""
          filterRole=""
          filterActive=""
          filterAdmin=""
          organizations={[{ id: 'org-1', name: 'Org 1' }]}
          roleOptions={[]}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept roleOptions prop', () => {
      const { container } = render(
        <UsersFilters
          filterEmail=""
          filterOrgId=""
          filterRole=""
          filterActive=""
          filterAdmin=""
          organizations={[]}
          roleOptions={['admin', 'user']}
          onFilterEmailChange={() => {}}
          onFilterOrgIdChange={() => {}}
          onFilterRoleChange={() => {}}
          onFilterActiveChange={() => {}}
          onFilterAdminChange={() => {}}
        />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})

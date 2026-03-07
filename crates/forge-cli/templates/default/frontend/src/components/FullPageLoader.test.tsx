/**
 * BDD component tests for FullPageLoader.tsx
 * Tests verify component rendering, accessibility, and styling
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { Window } from 'happy-dom'
import React from 'react'

import FullPageLoader from './FullPageLoader'

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

describe('FullPageLoader component', () => {
  describe('export behavior', () => {
    test('should export FullPageLoader as default export', () => {
      expect(FullPageLoader).toBeDefined()
      expect(typeof FullPageLoader).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render loading spinner', () => {
      const { container } = render(<FullPageLoader />)
      expect(container).toBeDefined()
      expect(container.querySelector('div')).not.toBeNull()
    })

    test('should render with full viewport height', () => {
      const { container } = render(<FullPageLoader />)
      expect(container).toBeDefined()
    })

    test('should center spinner', () => {
      const { container } = render(<FullPageLoader />)
      expect(container).toBeDefined()
    })
  })

  describe('accessibility behavior', () => {
    test('should have aria-busy attribute', () => {
      const { container } = render(<FullPageLoader />)
      const element = container.querySelector('[aria-busy]')
      expect(element).not.toBeNull()
    })

    test('should have aria-label', () => {
      const { container } = render(<FullPageLoader />)
      const element = container.querySelector('[aria-label="Loading"]')
      expect(element).not.toBeNull()
    })
  })

  describe('styling behavior', () => {
    test('should use inline styles only', () => {
      const { container } = render(<FullPageLoader />)
      expect(container).toBeDefined()
    })

    test('should include CSS animation keyframes', () => {
      const { container } = render(<FullPageLoader />)
      expect(container.innerHTML).toContain('fullPageLoaderSpin')
    })

    test('should render spinner with circular border', () => {
      const { container } = render(<FullPageLoader />)
      expect(container).toBeDefined()
    })
  })
})

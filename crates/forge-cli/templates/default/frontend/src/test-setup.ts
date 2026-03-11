/**
 * Test setup for React Testing Library and DOM environment.
 * Configures RTL to reduce verbose output and runs cleanup after each test.
 */
import { afterEach } from 'bun:test'
import { configure, cleanup } from '@testing-library/react'

configure({
  throwSuggestions: false,
})

afterEach(() => {
  cleanup()
})

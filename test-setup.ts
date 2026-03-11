/**
 * Test setup file for React Testing Library and DOM environment
 * Configures RTL to reduce verbose output and runs cleanup after each test
 * so DOM tests pass when run from monorepo root (shared document).
 */
import { afterEach } from 'bun:test'
import { configure, cleanup } from '@testing-library/react'

// Configure React Testing Library to reduce verbose error output
configure({
  throwSuggestions: false,
})

// Unmount React trees after each test so the next test sees a clean document
afterEach(() => {
  cleanup()
})

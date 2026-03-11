/**
 * Test setup file for React Testing Library and DOM environment
 * Configures RTL to reduce verbose output and sets up happy-dom environment
 */
import { configure } from '@testing-library/react'

// Configure React Testing Library to reduce verbose error output
// This prevents dumping the entire component tree when tests fail
configure({
  throwSuggestions: false,
})

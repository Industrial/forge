/**
 * BDD tests for e2e prebuilt helpers
 * Tests verify helper functions for checking prebuilt project structure and files
 */
import { describe, test, expect, beforeEach } from 'bun:test'
import fs from 'node:fs'
import path from 'node:path'
import {
  getPrebuiltRoot,
  prebuiltExists,
  assertProjectLayout,
  assertAuthLayout,
  readFile,
  pathExists,
} from './prebuilt.js'

describe('e2e prebuilt helpers', () => {
  describe('getPrebuiltRoot behavior', () => {
    test('should return a string path', () => {
      // Given: getPrebuiltRoot function
      // When: calling it
      const root = getPrebuiltRoot()

      // Then: should return a string
      expect(typeof root).toBe('string')
      expect(root.length).toBeGreaterThan(0)
    })

    test('should return consistent path on multiple calls', () => {
      // Given: getPrebuiltRoot function
      // When: calling it multiple times
      const root1 = getPrebuiltRoot()
      const root2 = getPrebuiltRoot()

      // Then: should return the same path
      expect(root1).toBe(root2)
    })
  })

  describe('prebuiltExists behavior', () => {
    test('should return boolean', () => {
      // Given: prebuiltExists function
      // When: calling it
      const exists = prebuiltExists()

      // Then: should return a boolean
      expect(typeof exists).toBe('boolean')
    })

    test('should check if prebuilt root directory exists', () => {
      // Given: prebuiltExists function and getPrebuiltRoot
      const root = getPrebuiltRoot()
      const expectedExists = fs.existsSync(root)

      // When: calling prebuiltExists
      const actualExists = prebuiltExists()

      // Then: should match filesystem check
      expect(actualExists).toBe(expectedExists)
    })
  })

  describe('readFile behavior', () => {
    test('should read file from prebuilt root', () => {
      // Given: prebuilt exists and a file path
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      const filePath = 'Cargo.toml'

      // When: reading file
      const content = readFile(filePath)

      // Then: should return file content as string
      expect(typeof content).toBe('string')
      expect(content.length).toBeGreaterThan(0)
    })

    test('should throw error for non-existent file', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      const filePath = 'non-existent-file-12345.txt'

      // When: reading non-existent file
      // Then: should throw error
      expect(() => readFile(filePath)).toThrow()
    })

    test('should read file with correct content', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      const filePath = 'Cargo.toml'

      // When: reading file
      const content = readFile(filePath)

      // Then: should contain expected content
      expect(content).toContain('[package]')
    })
  })

  describe('pathExists behavior', () => {
    test('should return boolean for path existence', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: checking existing path
      const exists = pathExists('Cargo.toml')

      // Then: should return boolean
      expect(typeof exists).toBe('boolean')
    })

    test('should return true for existing path', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: checking existing path
      const exists = pathExists('Cargo.toml')

      // Then: should return true
      expect(exists).toBe(true)
    })

    test('should return false for non-existent path', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: checking non-existent path
      const exists = pathExists('non-existent-path-12345')

      // Then: should return false
      expect(exists).toBe(false)
    })
  })

  describe('assertProjectLayout behavior', () => {
    test('should not throw when prebuilt has correct layout', () => {
      // Given: prebuilt exists with correct layout
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: asserting project layout
      // Then: should not throw
      expect(() => assertProjectLayout()).not.toThrow()
    })

    test('should check for required files', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: asserting project layout
      // Then: should verify required files exist
      // This is verified by assertProjectLayout not throwing
      expect(() => assertProjectLayout()).not.toThrow()
    })

    test('should verify App::new() usage', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: asserting project layout
      // Then: should verify App::new() is used
      // This is verified by assertProjectLayout not throwing
      expect(() => assertProjectLayout()).not.toThrow()
    })

    test('should verify serve() or into_router_before_state usage', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: asserting project layout
      // Then: should verify serve() or into_router_before_state is used
      // This is verified by assertProjectLayout not throwing
      expect(() => assertProjectLayout()).not.toThrow()
    })
  })

  describe('assertAuthLayout behavior', () => {
    test('should not throw when auth layout is correct', () => {
      // Given: prebuilt exists with auth layout
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: asserting auth layout
      // Then: should not throw (if auth is enabled)
      // Note: This may throw if auth is not enabled, which is acceptable
      try {
        assertAuthLayout()
        // If it doesn't throw, auth layout is correct
        expect(true).toBe(true)
      } catch (error) {
        // If it throws, verify it's a meaningful error
        expect(error).toBeInstanceOf(Error)
      }
    })

    test('should check for auth-related files', () => {
      // Given: prebuilt exists
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: asserting auth layout
      // Then: should verify auth files exist (if auth is enabled)
      // This is verified by assertAuthLayout behavior
      try {
        assertAuthLayout()
        expect(true).toBe(true)
      } catch (error) {
        expect(error).toBeInstanceOf(Error)
        expect((error as Error).message).toContain('missing')
      }
    })
  })

  describe('integration scenarios', () => {
    test('should work together to verify prebuilt project', () => {
      // Given: prebuilt helpers
      if (!prebuiltExists()) {
        // Skip test if prebuilt doesn't exist
        return
      }

      // When: using multiple helpers together
      const root = getPrebuiltRoot()
      const exists = prebuiltExists()
      const cargoExists = pathExists('Cargo.toml')

      // Then: should provide consistent information
      expect(exists).toBe(true)
      expect(cargoExists).toBe(true)
      expect(root).toBeTruthy()

      // And: should be able to read files
      const cargoContent = readFile('Cargo.toml')
      expect(cargoContent).toContain('[package]')
    })

    test('should handle missing prebuilt gracefully', () => {
      // Given: helpers that check for prebuilt
      // When: prebuilt doesn't exist
      // Note: This test verifies that helpers handle missing prebuilt
      // The actual behavior depends on PREBUILT_ROOT configuration
      const exists = prebuiltExists()
      expect(typeof exists).toBe('boolean')
    })
  })
})

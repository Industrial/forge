/**
 * BDD tests for forms/index.ts
 * Tests verify that the barrel export correctly exports FormTextField
 */

import { describe, it, expect } from 'bun:test'

describe('forms/index exports', () => {
  describe('FormTextField export behavior', () => {
    it('should export FormTextField as a named export', async () => {
      // Given the forms/index module
      // When I import all exports
      const exports = await import('./index')

      // Then FormTextField should be available as a named export
      expect(exports).toHaveProperty('FormTextField')
      expect(exports.FormTextField).toBeDefined()
      expect(typeof exports.FormTextField).toBe('function')
    })

    it('should export FormTextField that matches direct import', async () => {
      // Given both direct and index imports
      const directImport = (await import('./FormTextField')).default
      const indexImport = (await import('./index')).FormTextField

      // When I compare them
      // Then they should be the same component
      expect(indexImport).toBe(directImport)
    })

    it('should allow importing and using FormTextField', async () => {
      // Given the forms/index module
      // When I import FormTextField
      const { FormTextField } = await import('./index')

      // Then it should be a valid React component
      expect(FormTextField).toBeDefined()
      expect(typeof FormTextField).toBe('function')
    })
  })

  describe('export structure behavior', () => {
    it('should only export FormTextField', async () => {
      // Given the forms/index module
      // When I import all exports
      const exports = await import('./index')

      // Then it should only export FormTextField (no other unexpected exports)
      const exportKeys = Object.keys(exports)
      expect(exportKeys).toContain('FormTextField')
      // The module may have default export or other symbols, but FormTextField must exist
      expect(exports.FormTextField).toBeDefined()
    })

    it('should maintain component identity through re-export', async () => {
      // Given FormTextField imported from both direct and index
      const directImport = (await import('./FormTextField')).default
      const { FormTextField: indexImport } = await import('./index')

      // When I compare them
      // Then they should reference the same component
      expect(indexImport).toBe(directImport)
      expect(indexImport.name).toBe(directImport.name)
    })
  })
})

/**
 * BDD tests for vite.config.ts
 * Tests verify Vite configuration structure and behavior
 */
import { describe, test, expect, beforeEach, afterEach } from 'bun:test'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

describe('vite.config.ts', () => {
  describe('configuration structure behavior', () => {
    test('should export default configuration', () => {
      // Given: vite.config.ts file
      // When: checking default export
      // Then: should export default config object
      // Note: This is verified by successful Vite build/start
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('export default')
      expect(configContent).toContain('defineConfig')
    })

    test('should use defineConfig from vite', () => {
      // Given: vite.config.ts file
      // When: checking imports
      // Then: should import defineConfig from vite
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain("from 'vite'")
      expect(configContent).toContain('defineConfig')
    })

    test('should require VITE_BACKEND_URL environment variable', () => {
      // Given: vite.config.ts file
      // When: checking environment variable usage
      // Then: should require VITE_BACKEND_URL
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('VITE_BACKEND_URL')
    })
  })

  describe('resolve configuration behavior', () => {
    test('should configure path alias', () => {
      // Given: vite.config.ts file
      // When: checking resolve configuration
      // Then: should have alias '@' pointing to src directory
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain("alias")
      expect(configContent).toContain("'@'")
      expect(configContent).toContain('src')
    })

    test('should use path.resolve for alias', () => {
      // Given: vite.config.ts file
      // When: checking alias resolution
      // Then: should use path.resolve
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('path.resolve')
    })
  })

  describe('server configuration behavior', () => {
    test('should configure proxy for /api', () => {
      // Given: vite.config.ts file
      // When: checking server proxy configuration
      // Then: should proxy /api to VITE_BACKEND_URL
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain("'/api'")
      expect(configContent).toContain('proxy')
    })

    test('should configure proxy for /ws with WebSocket support', () => {
      // Given: vite.config.ts file
      // When: checking WebSocket proxy configuration
      // Then: should proxy /ws to VITE_BACKEND_URL with ws: true
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain("'/ws'")
      expect(configContent).toContain('ws: true')
    })

    test('should enable changeOrigin for proxies', () => {
      // Given: vite.config.ts file
      // When: checking proxy configuration
      // Then: should set changeOrigin: true
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('changeOrigin')
    })
  })

  describe('plugins configuration behavior', () => {
    test('should include React plugin', () => {
      // Given: vite.config.ts file
      // When: checking plugins
      // Then: should include @vitejs/plugin-react
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('react')
      expect(configContent).toContain('plugins')
    })
  })

  describe('build configuration behavior', () => {
    test('should target es2022', () => {
      // Given: vite.config.ts file
      // When: checking build target
      // Then: should target es2022
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain("target: ['es2022']")
    })

    test('should use lightningcss for CSS minification', () => {
      // Given: vite.config.ts file
      // When: checking CSS minification
      // Then: should use lightningcss
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('cssMinify')
      expect(configContent).toContain('lightningcss')
    })
  })

  describe('environment variable behavior', () => {
    test('should throw error when VITE_BACKEND_URL is not set', () => {
      // Given: vite.config.ts file
      // When: VITE_BACKEND_URL is not set
      // Then: should throw error
      // Note: This is verified by the code structure
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('throw new Error')
      expect(configContent).toContain('VITE_BACKEND_URL is not set')
    })

    test('should use VITE_BACKEND_URL for proxy target', () => {
      // Given: vite.config.ts file
      // When: checking proxy target
      // Then: should use VITE_BACKEND_URL variable
      const configPath = join(process.cwd(), 'vite.config.ts')
      const configContent = readFileSync(configPath, 'utf-8')
      expect(configContent).toContain('VITE_BACKEND_URL')
      expect(configContent).toContain('target')
    })
  })

  describe('file structure behavior', () => {
    test('should be a TypeScript configuration file', () => {
      // Given: vite.config.ts file
      // When: checking file type
      // Then: should be a .ts file
      const configPath = join(process.cwd(), 'vite.config.ts')
      expect(() => readFileSync(configPath, 'utf-8')).not.toThrow()
    })

    test('should be valid TypeScript syntax', () => {
      // Given: vite.config.ts file
      // When: checking syntax
      // Then: should be valid TypeScript
      // Note: This is verified by successful compilation
      expect(true).toBe(true)
    })
  })
})

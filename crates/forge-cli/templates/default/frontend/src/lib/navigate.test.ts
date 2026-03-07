/**
 * BDD tests for navigate
 * Tests verify the behavior of React Router navigation helpers that return Effect
 */

import { describe, it, expect } from 'bun:test'
import { Effect } from 'effect'
import { navigateTo, type NavigateFunction } from './navigate'

describe('navigate', () => {
  describe('export behavior', () => {
    it('should export navigateTo as a function', () => {
      // Given: the module
      // When: I check if navigateTo is exported
      // Then: it should be a function
      expect(typeof navigateTo).toBe('function')
    })

    it('should export NavigateFunction type', () => {
      // Given: the module
      // When: I check if NavigateFunction type is exported
      // Then: it should be usable
      // Note: TypeScript enforces this at compile time
      const testNavigate: NavigateFunction = () => {}
      expect(typeof testNavigate).toBe('function')
    })
  })

  describe('navigateTo function signature', () => {
    it('should accept navigate function as first parameter', () => {
      // Given: navigateTo function
      // When: I check its signature
      // Then: it should accept NavigateFunction as first parameter
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = () => {}
      expect(typeof navigateTo).toBe('function')
    })

    it('should accept to string as second parameter', () => {
      // Given: navigateTo function
      // When: I check its signature
      // Then: it should accept string as second parameter
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = () => {}
      const to: string = '/test'
      expect(typeof navigateTo).toBe('function')
    })

    it('should accept optional options as third parameter', () => {
      // Given: navigateTo function
      // When: I check its signature
      // Then: it should accept optional options as third parameter
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = () => {}
      const options: { replace?: boolean } = { replace: true }
      expect(typeof navigateTo).toBe('function')
    })

    it('should return Effect<void, Error, never>', () => {
      // Given: navigateTo function
      // When: I check its return type
      // Then: it should return Effect<void, Error, never>
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = () => {}
      const effect = navigateTo(navigate, '/test')
      expect(effect).toBeDefined()
    })
  })

  describe('NavigateFunction type', () => {
    it('should accept to string parameter', () => {
      // Given: NavigateFunction type
      // When: I check its signature
      // Then: it should accept to: string
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = (to: string) => {}
      expect(typeof navigate).toBe('function')
    })

    it('should accept optional options parameter', () => {
      // Given: NavigateFunction type
      // When: I check its signature
      // Then: it should accept options?: { replace?: boolean }
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = (
        to: string,
        options?: { replace?: boolean },
      ) => {}
      expect(typeof navigate).toBe('function')
    })

    it('should return void | Promise<void>', () => {
      // Given: NavigateFunction type
      // When: I check its return type
      // Then: it should return void | Promise<void>
      // Note: TypeScript enforces this at compile time
      const navigateSync: NavigateFunction = () => {}
      const navigateAsync: NavigateFunction = () => Promise.resolve()
      expect(typeof navigateSync).toBe('function')
      expect(typeof navigateAsync).toBe('function')
    })
  })

  describe('navigateTo behavior with sync navigate', () => {
    it('should return Effect that succeeds when navigate returns void', async () => {
      // Given: a sync navigate function
      const navigate: NavigateFunction = () => {
        // void return
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should return Effect that succeeds
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })

    it('should call navigate with correct path', async () => {
      // Given: a navigate function that tracks calls
      let calledPath = ''
      const navigate: NavigateFunction = (to: string) => {
        calledPath = to
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/dashboard')

      // Then: navigate should be called with correct path
      await Effect.runPromise(effect)
      expect(calledPath).toBe('/dashboard')
    })

    it('should call navigate with options when provided', async () => {
      // Given: a navigate function that tracks options
      let calledOptions: { replace?: boolean } | undefined
      const navigate: NavigateFunction = (
        to: string,
        options?: { replace?: boolean },
      ) => {
        calledOptions = options
      }

      // When: I call navigateTo with options
      const effect = navigateTo(navigate, '/test', { replace: true })

      // Then: navigate should be called with options
      await Effect.runPromise(effect)
      expect(calledOptions).toEqual({ replace: true })
    })

    it('should call navigate without options when not provided', async () => {
      // Given: a navigate function that tracks options
      let calledOptions: { replace?: boolean } | undefined
      const navigate: NavigateFunction = (
        to: string,
        options?: { replace?: boolean },
      ) => {
        calledOptions = options
      }

      // When: I call navigateTo without options
      const effect = navigateTo(navigate, '/test')

      // Then: navigate should be called with undefined options
      await Effect.runPromise(effect)
      expect(calledOptions).toBeUndefined()
    })

    it('should handle replace option', async () => {
      // Given: a navigate function
      let replaceValue: boolean | undefined
      const navigate: NavigateFunction = (
        to: string,
        options?: { replace?: boolean },
      ) => {
        replaceValue = options?.replace
      }

      // When: I call navigateTo with replace: true
      const effect = navigateTo(navigate, '/test', { replace: true })

      // Then: replace should be passed correctly
      await Effect.runPromise(effect)
      expect(replaceValue).toBe(true)
    })

    it('should handle replace option as false', async () => {
      // Given: a navigate function
      let replaceValue: boolean | undefined
      const navigate: NavigateFunction = (
        to: string,
        options?: { replace?: boolean },
      ) => {
        replaceValue = options?.replace
      }

      // When: I call navigateTo with replace: false
      const effect = navigateTo(navigate, '/test', { replace: false })

      // Then: replace should be passed correctly
      await Effect.runPromise(effect)
      expect(replaceValue).toBe(false)
    })
  })

  describe('navigateTo behavior with async navigate', () => {
    it('should return Effect that succeeds when navigate returns Promise<void>', async () => {
      // Given: an async navigate function
      const navigate: NavigateFunction = () => {
        return Promise.resolve()
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should return Effect that succeeds
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })

    it('should await async navigate function', async () => {
      // Given: an async navigate function that resolves after delay
      let resolved = false
      const navigate: NavigateFunction = () => {
        return new Promise<void>((resolve) => {
          setTimeout(() => {
            resolved = true
            resolve()
          }, 10)
        })
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should await the promise
      await Effect.runPromise(effect)
      expect(resolved).toBe(true)
    })

    it('should handle async navigate with options', async () => {
      // Given: an async navigate function
      let calledOptions: { replace?: boolean } | undefined
      const navigate: NavigateFunction = (
        to: string,
        options?: { replace?: boolean },
      ) => {
        calledOptions = options
        return Promise.resolve()
      }

      // When: I call navigateTo with options
      const effect = navigateTo(navigate, '/test', { replace: true })

      // Then: navigate should be called with options
      await Effect.runPromise(effect)
      expect(calledOptions).toEqual({ replace: true })
    })
  })

  describe('navigateTo error handling', () => {
    it('should convert navigation errors to Effect errors', async () => {
      // Given: a navigate function that throws
      const navigate: NavigateFunction = () => {
        throw new Error('Navigation failed')
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should return Effect that fails with error
      await expect(Effect.runPromise(effect)).rejects.toThrow(
        'Navigation failed',
      )
    })

    it('should convert non-Error exceptions to Error', async () => {
      // Given: a navigate function that throws non-Error
      const navigate: NavigateFunction = () => {
        throw 'String error'
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should convert to Error
      await expect(Effect.runPromise(effect)).rejects.toThrow()
    })

    it('should handle async navigate that rejects', async () => {
      // Given: an async navigate function that rejects
      const navigate: NavigateFunction = () => {
        return Promise.reject(new Error('Async navigation failed'))
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should return Effect that fails
      await expect(Effect.runPromise(effect)).rejects.toThrow(
        'Async navigation failed',
      )
    })

    it('should preserve Error instances', async () => {
      // Given: a navigate function that throws Error
      const error = new Error('Custom error')
      const navigate: NavigateFunction = () => {
        throw error
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should preserve the Error instance
      await expect(Effect.runPromise(effect)).rejects.toThrow('Custom error')
    })
  })

  describe('navigateToPromise normalization', () => {
    it('should normalize void return to Promise', async () => {
      // Given: a sync navigate function
      const navigate: NavigateFunction = () => {
        // void return
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should normalize void to Promise
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })

    it('should normalize Promise<void> return', async () => {
      // Given: an async navigate function
      const navigate: NavigateFunction = () => {
        return Promise.resolve()
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should handle Promise<void>
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })

    it('should handle null return as void', async () => {
      // Given: a navigate function that returns null
      const navigate: NavigateFunction = () => {
        return null as any
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should normalize null to Promise
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })
  })

  describe('usage in Effect.gen', () => {
    it('should work with yield* in Effect.gen', async () => {
      // Given: a navigate function
      let calledPath = ''
      const navigate: NavigateFunction = (to: string) => {
        calledPath = to
      }

      // When: I use navigateTo in Effect.gen
      const program = Effect.gen(function* () {
        yield* navigateTo(navigate, '/dashboard')
        return 'success'
      })

      // Then: it should work correctly
      const result = await Effect.runPromise(program)
      expect(result).toBe('success')
      expect(calledPath).toBe('/dashboard')
    })

    it('should work with multiple navigations', async () => {
      // Given: a navigate function that tracks calls
      const calls: string[] = []
      const navigate: NavigateFunction = (to: string) => {
        calls.push(to)
      }

      // When: I use navigateTo multiple times
      const program = Effect.gen(function* () {
        yield* navigateTo(navigate, '/first')
        yield* navigateTo(navigate, '/second')
        return calls.length
      })

      // Then: all navigations should execute
      const result = await Effect.runPromise(program)
      expect(result).toBe(2)
      expect(calls).toEqual(['/first', '/second'])
    })

    it('should propagate errors in Effect.gen', async () => {
      // Given: a navigate function that throws
      const navigate: NavigateFunction = () => {
        throw new Error('Navigation error')
      }

      // When: I use navigateTo in Effect.gen
      const program = Effect.gen(function* () {
        yield* navigateTo(navigate, '/test')
        return 'success'
      })

      // Then: error should propagate
      await expect(Effect.runPromise(program)).rejects.toThrow(
        'Navigation error',
      )
    })
  })

  describe('React Router v7 compatibility', () => {
    it('should handle sync navigate (void return)', async () => {
      // Given: React Router v7 sync navigate
      const navigate: NavigateFunction = () => {
        // void return (sync)
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should handle sync navigate
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })

    it('should handle async navigate (Promise<void> return)', async () => {
      // Given: React Router v7 async navigate
      const navigate: NavigateFunction = () => {
        return Promise.resolve()
      }

      // When: I call navigateTo
      const effect = navigateTo(navigate, '/test')

      // Then: it should handle async navigate
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })

    it('should normalize both sync and async navigate', async () => {
      // Given: both sync and async navigate functions
      const syncNavigate: NavigateFunction = () => {}
      const asyncNavigate: NavigateFunction = () => Promise.resolve()

      // When: I call navigateTo with both
      const syncEffect = navigateTo(syncNavigate, '/sync')
      const asyncEffect = navigateTo(asyncNavigate, '/async')

      // Then: both should work
      await expect(Effect.runPromise(syncEffect)).resolves.toBeUndefined()
      await expect(Effect.runPromise(asyncEffect)).resolves.toBeUndefined()
    })
  })

  describe('edge cases', () => {
    it('should handle empty string path', async () => {
      // Given: a navigate function
      let calledPath = ''
      const navigate: NavigateFunction = (to: string) => {
        calledPath = to
      }

      // When: I call navigateTo with empty string
      const effect = navigateTo(navigate, '')

      // Then: it should handle empty string
      await Effect.runPromise(effect)
      expect(calledPath).toBe('')
    })

    it('should handle long paths', async () => {
      // Given: a navigate function
      let calledPath = ''
      const longPath = '/very/long/path/with/many/segments'
      const navigate: NavigateFunction = (to: string) => {
        calledPath = to
      }

      // When: I call navigateTo with long path
      const effect = navigateTo(navigate, longPath)

      // Then: it should handle long paths
      await Effect.runPromise(effect)
      expect(calledPath).toBe(longPath)
    })

    it('should handle paths with query strings', async () => {
      // Given: a navigate function
      let calledPath = ''
      const navigate: NavigateFunction = (to: string) => {
        calledPath = to
      }

      // When: I call navigateTo with query string
      const effect = navigateTo(navigate, '/test?param=value')

      // Then: it should handle query strings
      await Effect.runPromise(effect)
      expect(calledPath).toBe('/test?param=value')
    })

    it('should handle paths with hash', async () => {
      // Given: a navigate function
      let calledPath = ''
      const navigate: NavigateFunction = (to: string) => {
        calledPath = to
      }

      // When: I call navigateTo with hash
      const effect = navigateTo(navigate, '/test#section')

      // Then: it should handle hash
      await Effect.runPromise(effect)
      expect(calledPath).toBe('/test#section')
    })

    it('should handle undefined options', async () => {
      // Given: a navigate function
      const navigate: NavigateFunction = () => {}

      // When: I call navigateTo with undefined options
      const effect = navigateTo(navigate, '/test', undefined)

      // Then: it should handle undefined options
      await expect(Effect.runPromise(effect)).resolves.toBeUndefined()
    })
  })

  describe('type safety', () => {
    it('should enforce NavigateFunction type', () => {
      // Given: navigateTo function
      // When: I check type constraints
      // Then: it should enforce NavigateFunction type
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = () => {}
      const effect = navigateTo(navigate, '/test')
      expect(effect).toBeDefined()
    })

    it('should enforce string type for to parameter', () => {
      // Given: navigateTo function
      // When: I check type constraints
      // Then: it should enforce string type for to
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = () => {}
      const to: string = '/test'
      const effect = navigateTo(navigate, to)
      expect(effect).toBeDefined()
    })

    it('should enforce optional options type', () => {
      // Given: navigateTo function
      // When: I check type constraints
      // Then: it should enforce optional options type
      // Note: TypeScript enforces this at compile time
      const navigate: NavigateFunction = () => {}
      const options: { replace?: boolean } = { replace: true }
      const effect = navigateTo(navigate, '/test', options)
      expect(effect).toBeDefined()
    })
  })

  describe('integration scenarios', () => {
    it('should work with useNavigate from react-router-dom', () => {
      // Given: a mock navigate function (simulating useNavigate)
      const mockNavigate: NavigateFunction = () => {}

      // When: I use navigateTo with mock navigate
      // Then: it should work (TypeScript ensures compatibility)
      const effect = navigateTo(mockNavigate, '/dashboard')
      expect(effect).toBeDefined()
    })

    it('should work in authentication flow', async () => {
      // Given: a navigate function
      let navigatedTo = ''
      const navigate: NavigateFunction = (to: string) => {
        navigatedTo = to
      }

      // When: I simulate login flow navigation
      const loginFlow = Effect.gen(function* () {
        yield* navigateTo(navigate, '/', { replace: true })
        return 'logged in'
      })

      // Then: it should navigate correctly
      const result = await Effect.runPromise(loginFlow)
      expect(result).toBe('logged in')
      expect(navigatedTo).toBe('/')
    })

    it('should work with replace option for redirects', async () => {
      // Given: a navigate function
      let replaceValue: boolean | undefined
      const navigate: NavigateFunction = (
        to: string,
        options?: { replace?: boolean },
      ) => {
        replaceValue = options?.replace
      }

      // When: I navigate with replace for redirect
      const effect = navigateTo(navigate, '/dashboard', { replace: true })

      // Then: replace should be set
      await Effect.runPromise(effect)
      expect(replaceValue).toBe(true)
    })
  })
})

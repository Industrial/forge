/**
 * BDD-style unit tests for API types using bun:test and Effect Schema.
 * Tests follow Given-When-Then pattern and are organized by behavior.
 */

import { describe, it, expect } from 'bun:test'
import { Schema } from 'effect'
import {
  ListQueryParamsSchema,
  ListResponseSchema,
  GetResponseSchema,
  RpcSubscribeRequestSchema,
  RpcSubscribeResultSchema,
  RpcResponseSchema,
  AuthMeBodySchema,
  LoginResponseSchema,
  RegisterResponseSchema,
  ApiErrorBodySchema,
} from './types'

// ---------------------------------------------------------------------------
// ListQueryParams Behavior
// ---------------------------------------------------------------------------

describe('ListQueryParams Behavior', () => {
  describe('should validate list query parameters', () => {
    it('should accept valid list query params with all fields', () => {
      // Given: a valid list query params object with all fields
      const validParams = {
        filter: 'name eq "test"',
        sort: 'created_at',
        order: 'desc',
        offset: 0,
        limit: 10,
      }

      // When: decoding the params
      const result = Schema.decodeUnknownSync(ListQueryParamsSchema)(
        validParams,
      )

      // Then: should decode successfully
      expect(result).toEqual(validParams)
    })

    it('should accept list query params with only filter', () => {
      // Given: list query params with only filter field
      const params = {
        filter: 'name eq "test"',
      }

      // When: decoding the params
      const result = Schema.decodeUnknownSync(ListQueryParamsSchema)(params)

      // Then: should decode successfully
      expect(result.filter).toBe('name eq "test"')
      expect(result.sort).toBeUndefined()
    })

    it('should accept list query params with no fields', () => {
      // Given: an empty object
      const params = {}

      // When: decoding the params
      const result = Schema.decodeUnknownSync(ListQueryParamsSchema)(params)

      // Then: should decode successfully with all optional fields undefined
      expect(result.filter).toBeUndefined()
      expect(result.sort).toBeUndefined()
      expect(result.order).toBeUndefined()
      expect(result.offset).toBeUndefined()
      expect(result.limit).toBeUndefined()
    })

    it('should reject invalid offset type', () => {
      // Given: list query params with invalid offset type
      const invalidParams = {
        offset: 'not-a-number',
      }

      // When: decoding the params
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(ListQueryParamsSchema)(invalidParams)
      }).toThrow()
    })

    it('should reject invalid limit type', () => {
      // Given: list query params with invalid limit type
      const invalidParams = {
        limit: 'not-a-number',
      }

      // When: decoding the params
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(ListQueryParamsSchema)(invalidParams)
      }).toThrow()
    })
  })
})

// ---------------------------------------------------------------------------
// ListResponse Behavior
// ---------------------------------------------------------------------------

describe('ListResponse Behavior', () => {
  describe('should validate list response structure', () => {
    it('should accept valid list response with data array', () => {
      // Given: a valid list response with string items
      const itemSchema = Schema.String
      const responseSchema = ListResponseSchema(itemSchema)
      const validResponse = {
        data: ['item1', 'item2', 'item3'],
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(validResponse)

      // Then: should decode successfully
      expect(result.data).toEqual(['item1', 'item2', 'item3'])
    })

    it('should accept empty data array', () => {
      // Given: a list response with empty data array
      const itemSchema = Schema.String
      const responseSchema = ListResponseSchema(itemSchema)
      const emptyResponse = {
        data: [],
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(emptyResponse)

      // Then: should decode successfully
      expect(result.data).toEqual([])
    })

    it('should reject response without data field', () => {
      // Given: a response object without data field
      const itemSchema = Schema.String
      const responseSchema = ListResponseSchema(itemSchema)
      const invalidResponse = {}

      // When: decoding the response
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(responseSchema)(invalidResponse)
      }).toThrow()
    })

    it('should validate nested item schemas', () => {
      // Given: a list response with nested object items
      const itemSchema = Schema.Struct({
        id: Schema.String,
        name: Schema.String,
      })
      const responseSchema = ListResponseSchema(itemSchema)
      const validResponse = {
        data: [
          { id: '1', name: 'Item 1' },
          { id: '2', name: 'Item 2' },
        ],
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(validResponse)

      // Then: should decode successfully with validated items
      expect(result.data).toHaveLength(2)
      expect(result.data[0].id).toBe('1')
      expect(result.data[0].name).toBe('Item 1')
    })
  })
})

// ---------------------------------------------------------------------------
// GetResponse Behavior
// ---------------------------------------------------------------------------

describe('GetResponse Behavior', () => {
  describe('should validate get response structure', () => {
    it('should accept valid get response with data object', () => {
      // Given: a valid get response with string data
      const itemSchema = Schema.String
      const responseSchema = GetResponseSchema(itemSchema)
      const validResponse = {
        data: 'test-value',
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(validResponse)

      // Then: should decode successfully
      expect(result.data).toBe('test-value')
    })

    it('should accept get response with nested object', () => {
      // Given: a get response with nested object data
      const itemSchema = Schema.Struct({
        id: Schema.String,
        name: Schema.String,
      })
      const responseSchema = GetResponseSchema(itemSchema)
      const validResponse = {
        data: {
          id: '123',
          name: 'Test Item',
        },
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(validResponse)

      // Then: should decode successfully
      expect(result.data.id).toBe('123')
      expect(result.data.name).toBe('Test Item')
    })

    it('should reject response without data field', () => {
      // Given: a response object without data field
      const itemSchema = Schema.String
      const responseSchema = GetResponseSchema(itemSchema)
      const invalidResponse = {}

      // When: decoding the response
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(responseSchema)(invalidResponse)
      }).toThrow()
    })
  })
})

// ---------------------------------------------------------------------------
// RpcSubscribeRequest Behavior
// ---------------------------------------------------------------------------

describe('RpcSubscribeRequest Behavior', () => {
  describe('should validate RPC subscribe request', () => {
    it('should accept valid subscribe request with all fields', () => {
      // Given: a valid subscribe request with method, entity_id, and params
      const validRequest = {
        method: 'subscribe' as const,
        entity_id: 'users',
        params: {
          filter: 'name eq "test"',
          sort: 'created_at',
          offset: 0,
          limit: 10,
        },
      }

      // When: decoding the request
      const result = Schema.decodeUnknownSync(RpcSubscribeRequestSchema)(
        validRequest,
      )

      // Then: should decode successfully
      expect(result.method).toBe('subscribe')
      expect(result.entity_id).toBe('users')
      expect(result.params?.filter).toBe('name eq "test"')
    })

    it('should accept subscribe request without params', () => {
      // Given: a subscribe request without params
      const request = {
        method: 'subscribe' as const,
        entity_id: 'users',
      }

      // When: decoding the request
      const result = Schema.decodeUnknownSync(RpcSubscribeRequestSchema)(
        request,
      )

      // Then: should decode successfully
      expect(result.method).toBe('subscribe')
      expect(result.entity_id).toBe('users')
      expect(result.params).toBeUndefined()
    })

    it('should reject request with invalid method', () => {
      // Given: a request with invalid method value
      const invalidRequest = {
        method: 'invalid',
        entity_id: 'users',
      }

      // When: decoding the request
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(RpcSubscribeRequestSchema)(invalidRequest)
      }).toThrow()
    })

    it('should reject request without entity_id', () => {
      // Given: a request without entity_id
      const invalidRequest = {
        method: 'subscribe' as const,
      }

      // When: decoding the request
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(RpcSubscribeRequestSchema)(invalidRequest)
      }).toThrow()
    })
  })
})

// ---------------------------------------------------------------------------
// RpcSubscribeResult Behavior
// ---------------------------------------------------------------------------

describe('RpcSubscribeResult Behavior', () => {
  describe('should validate RPC subscribe result', () => {
    it('should accept valid subscribe result with subscription_id', () => {
      // Given: a valid subscribe result
      const validResult = {
        subscription_id: '123e4567-e89b-12d3-a456-426614174000',
      }

      // When: decoding the result
      const result = Schema.decodeUnknownSync(RpcSubscribeResultSchema)(
        validResult,
      )

      // Then: should decode successfully
      expect(result.subscription_id).toBe(
        '123e4567-e89b-12d3-a456-426614174000',
      )
    })

    it('should reject result without subscription_id', () => {
      // Given: a result without subscription_id
      const invalidResult = {}

      // When: decoding the result
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(RpcSubscribeResultSchema)(invalidResult)
      }).toThrow()
    })
  })
})

// ---------------------------------------------------------------------------
// RpcResponse Behavior
// ---------------------------------------------------------------------------

describe('RpcResponse Behavior', () => {
  describe('should validate RPC response structure', () => {
    it('should accept valid RPC response with result', () => {
      // Given: a valid RPC response with result
      const resultSchema = Schema.Struct({ id: Schema.String })
      const responseSchema = RpcResponseSchema(resultSchema)
      const validResponse = {
        result: { id: '123' },
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(validResponse)

      // Then: should decode successfully
      expect(result.result?.id).toBe('123')
      expect(result.error).toBeUndefined()
    })

    it('should accept valid RPC response with error', () => {
      // Given: a valid RPC response with error
      const resultSchema = Schema.Struct({ id: Schema.String })
      const responseSchema = RpcResponseSchema(resultSchema)
      const errorResponse = {
        error: {
          message: 'Something went wrong',
        },
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(errorResponse)

      // Then: should decode successfully
      expect(result.error?.message).toBe('Something went wrong')
      expect(result.result).toBeUndefined()
    })

    it('should accept RPC response with error without message', () => {
      // Given: an RPC response with error but no message
      const resultSchema = Schema.Struct({ id: Schema.String })
      const responseSchema = RpcResponseSchema(resultSchema)
      const errorResponse = {
        error: {},
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(errorResponse)

      // Then: should decode successfully
      expect(result.error).toBeDefined()
      expect(result.error?.message).toBeUndefined()
    })

    it('should accept empty RPC response', () => {
      // Given: an empty RPC response
      const resultSchema = Schema.Struct({ id: Schema.String })
      const responseSchema = RpcResponseSchema(resultSchema)
      const emptyResponse = {}

      // When: decoding the response
      const result = Schema.decodeUnknownSync(responseSchema)(emptyResponse)

      // Then: should decode successfully (both result and error are optional)
      expect(result.result).toBeUndefined()
      expect(result.error).toBeUndefined()
    })
  })
})

// ---------------------------------------------------------------------------
// AuthMeBody Behavior
// ---------------------------------------------------------------------------

describe('AuthMeBody Behavior', () => {
  describe('should validate auth me body structure', () => {
    it('should accept valid auth me body with all fields', () => {
      // Given: a valid auth me body with all fields
      const validBody = {
        user: {
          id: '123',
          email: 'user@example.com',
          permissions: ['read', 'write'],
        },
        permissions: ['read', 'write'],
        profiles: [{ id: '1', name: 'Profile 1' }],
        flash: { message: 'Success' },
      }

      // When: decoding the body
      const result = Schema.decodeUnknownSync(AuthMeBodySchema)(validBody)

      // Then: should decode successfully
      expect(result.user?.id).toBe('123')
      expect(result.user?.email).toBe('user@example.com')
      expect(result.permissions).toEqual(['read', 'write'])
    })

    it('should accept auth me body with minimal fields', () => {
      // Given: an auth me body with minimal fields
      const minimalBody = {
        user: {
          id: '123',
        },
      }

      // When: decoding the body
      const result = Schema.decodeUnknownSync(AuthMeBodySchema)(minimalBody)

      // Then: should decode successfully
      expect(result.user?.id).toBe('123')
      expect(result.user?.email).toBeUndefined()
    })

    it('should accept empty auth me body', () => {
      // Given: an empty auth me body
      const emptyBody = {}

      // When: decoding the body
      const result = Schema.decodeUnknownSync(AuthMeBodySchema)(emptyBody)

      // Then: should decode successfully (all fields are optional)
      expect(result.user).toBeUndefined()
      expect(result.permissions).toBeUndefined()
    })
  })
})

// ---------------------------------------------------------------------------
// LoginResponse Behavior
// ---------------------------------------------------------------------------

describe('LoginResponse Behavior', () => {
  describe('should validate login response structure', () => {
    it('should accept valid login response with all fields', () => {
      // Given: a valid login response
      const validResponse = {
        ok: true,
        token: 'jwt-token-here',
      }

      // When: decoding the response
      const result =
        Schema.decodeUnknownSync(LoginResponseSchema)(validResponse)

      // Then: should decode successfully
      expect(result.ok).toBe(true)
      expect(result.token).toBe('jwt-token-here')
    })

    it('should reject login response without ok field', () => {
      // Given: a login response without ok field
      const invalidResponse = {
        token: 'jwt-token-here',
      }

      // When: decoding the response
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(LoginResponseSchema)(invalidResponse)
      }).toThrow()
    })

    it('should reject login response without token field', () => {
      // Given: a login response without token field
      const invalidResponse = {
        ok: true,
      }

      // When: decoding the response
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(LoginResponseSchema)(invalidResponse)
      }).toThrow()
    })
  })
})

// ---------------------------------------------------------------------------
// RegisterResponse Behavior
// ---------------------------------------------------------------------------

describe('RegisterResponse Behavior', () => {
  describe('should validate register response structure', () => {
    it('should accept valid register response with all fields', () => {
      // Given: a valid register response
      const validResponse = {
        ok: true,
        message: 'Registration successful',
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(RegisterResponseSchema)(
        validResponse,
      )

      // Then: should decode successfully
      expect(result.ok).toBe(true)
      expect(result.message).toBe('Registration successful')
    })

    it('should accept register response with only ok field', () => {
      // Given: a register response with only ok field
      const response = {
        ok: true,
      }

      // When: decoding the response
      const result = Schema.decodeUnknownSync(RegisterResponseSchema)(response)

      // Then: should decode successfully
      expect(result.ok).toBe(true)
      expect(result.message).toBeUndefined()
    })

    it('should accept empty register response', () => {
      // Given: an empty register response
      const emptyResponse = {}

      // When: decoding the response
      const result = Schema.decodeUnknownSync(RegisterResponseSchema)(
        emptyResponse,
      )

      // Then: should decode successfully (all fields are optional)
      expect(result.ok).toBeUndefined()
      expect(result.message).toBeUndefined()
    })
  })
})

// ---------------------------------------------------------------------------
// ApiErrorBody Behavior
// ---------------------------------------------------------------------------

describe('ApiErrorBody Behavior', () => {
  describe('should validate API error body structure', () => {
    it('should accept error body with message field', () => {
      // Given: an error body with message field
      const errorBody = {
        message: 'An error occurred',
      }

      // When: decoding the error body
      const result = Schema.decodeUnknownSync(ApiErrorBodySchema)(errorBody)

      // Then: should decode successfully
      expect(result.message).toBe('An error occurred')
    })

    it('should accept error body with error string', () => {
      // Given: an error body with error as string
      const errorBody = {
        error: 'Error message string',
      }

      // When: decoding the error body
      const result = Schema.decodeUnknownSync(ApiErrorBodySchema)(errorBody)

      // Then: should decode successfully
      expect(result.error).toBe('Error message string')
    })

    it('should accept error body with error object', () => {
      // Given: an error body with error as object
      const errorBody = {
        error: {
          message: 'Error message in object',
        },
      }

      // When: decoding the error body
      const result = Schema.decodeUnknownSync(ApiErrorBodySchema)(errorBody)

      // Then: should decode successfully
      expect(result.error).toEqual({ message: 'Error message in object' })
    })

    it('should accept empty error body', () => {
      // Given: an empty error body
      const emptyBody = {}

      // When: decoding the error body
      const result = Schema.decodeUnknownSync(ApiErrorBodySchema)(emptyBody)

      // Then: should decode successfully (all fields are optional)
      expect(result.message).toBeUndefined()
      expect(result.error).toBeUndefined()
    })

    it('should accept error body with both message and error', () => {
      // Given: an error body with both message and error fields
      const errorBody = {
        message: 'Top-level message',
        error: 'Error detail',
      }

      // When: decoding the error body
      const result = Schema.decodeUnknownSync(ApiErrorBodySchema)(errorBody)

      // Then: should decode successfully
      expect(result.message).toBe('Top-level message')
      expect(result.error).toBe('Error detail')
    })
  })
})

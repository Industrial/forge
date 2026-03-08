/**
 * BDD tests for useForm hook
 * Tests verify hook behavior, form state management, validation, and error handling
 * Tests follow Given-When-Then pattern.
 */
import { describe, test, expect } from 'bun:test'
import { renderHook, act } from '@testing-library/react'
import { Schema, Effect, Either } from 'effect'
import { useForm, type FieldError, type UseFormConfig } from './useForm'

// Test schemas
const SimpleStringSchema = Schema.Struct({
  name: Schema.String,
})

const RequiredStringSchema = Schema.Struct({
  name: Schema.String.pipe(
    Schema.filter((s: string) => s.length > 0, {
      message: () => 'Name is required',
    }),
  ),
})

const EmailSchema = Schema.Struct({
  email: Schema.String.pipe(
    Schema.filter((s: string) => s.length > 0, {
      message: () => 'Email is required',
    }),
  ).pipe(
    Schema.pattern(/^[^\s@]+@[^\s@]+\.[^\s@]+$/, {
      message: () => 'Invalid email format',
    }),
  ),
})

const MultiFieldSchema = Schema.Struct({
  name: Schema.String.pipe(
    Schema.filter((s: string) => s.length > 0, {
      message: () => 'Name is required',
    }),
  ),
  age: Schema.Number.pipe(
    Schema.filter((n: number) => n >= 0, {
      message: () => 'Age must be non-negative',
    }),
  ),
})

describe('useForm', () => {
  describe('export behavior', () => {
    test('should export useForm as a function', () => {
      // Given: the module
      // When: checking the export
      // Then: should be a function
      expect(typeof useForm).toBe('function')
    })

    test('should export FieldError interface', () => {
      // Given: the module
      // When: checking FieldError type
      // Then: should be usable as a type
      const error: FieldError = { field: 'test', message: 'test error' }
      expect(error.field).toBe('test')
      expect(error.message).toBe('test error')
    })

    test('should export UseFormConfig interface', () => {
      // Given: the module
      // When: checking UseFormConfig type
      // Then: should be usable as a type
      const config: UseFormConfig<{ name: string }> = {
        schema: SimpleStringSchema,
        initialValues: { name: '' },
      }
      expect(config.schema).toBeDefined()
      expect(config.initialValues).toBeDefined()
    })
  })

  describe('initialization behavior', () => {
    test('should initialize with provided initial values', () => {
      // Given: initial values
      const initialValues = { name: 'John' }

      // When: initializing the hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues,
        }),
      )

      // Then: form state should have initial values
      expect(result.current.formState.values).toEqual(initialValues)
      expect(result.current.formState.errors).toEqual([])
      expect(result.current.formState.isValid).toBe(false)
      expect(result.current.formState.touched).toEqual({})
    })

    test('should initialize with empty errors array', () => {
      // Given: initial values
      const initialValues = { name: '' }

      // When: initializing the hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues,
        }),
      )

      // Then: errors should be empty
      expect(result.current.formState.errors).toEqual([])
    })

    test('should initialize with isValid false', () => {
      // Given: initial values
      const initialValues = { name: '' }

      // When: initializing the hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues,
        }),
      )

      // Then: isValid should be false
      expect(result.current.formState.isValid).toBe(false)
    })

    test('should initialize with empty touched object', () => {
      // Given: initial values
      const initialValues = { name: '' }

      // When: initializing the hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues,
        }),
      )

      // Then: touched should be empty
      expect(result.current.formState.touched).toEqual({})
    })
  })

  describe('setFieldValue behavior', () => {
    test('should update field value', () => {
      // Given: initialized hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: setting field value
      act(() => {
        result.current.setFieldValue('name', 'Jane')
      })

      // Then: value should be updated
      expect(result.current.formState.values.name).toBe('Jane')
    })

    test('should preserve other field values when updating one', () => {
      // Given: hook with multiple fields
      const { result } = renderHook(() =>
        useForm({
          schema: MultiFieldSchema,
          initialValues: { name: 'John', age: 25 },
        }),
      )

      // When: updating one field
      act(() => {
        result.current.setFieldValue('name', 'Jane')
      })

      // Then: other fields should be preserved
      expect(result.current.formState.values.name).toBe('Jane')
      expect(result.current.formState.values.age).toBe(25)
    })

    test('should not trigger validation when updating value', () => {
      // Given: hook with validation schema
      const { result } = renderHook(() =>
        useForm({
          schema: RequiredStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: setting invalid value
      act(() => {
        result.current.setFieldValue('name', '')
      })

      // Then: errors should remain empty (validation not triggered)
      expect(result.current.formState.errors).toEqual([])
    })
  })

  describe('setFieldTouched behavior', () => {
    test('should mark field as touched', () => {
      // Given: initialized hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: marking field as touched
      act(() => {
        result.current.setFieldTouched('name')
      })

      // Then: field should be marked as touched
      expect(result.current.formState.touched.name).toBe(true)
    })

    test('should preserve other touched fields', () => {
      // Given: hook with multiple fields
      const { result } = renderHook(() =>
        useForm({
          schema: MultiFieldSchema,
          initialValues: { name: '', age: 0 },
        }),
      )

      // When: marking one field as touched, then another
      act(() => {
        result.current.setFieldTouched('name')
      })
      act(() => {
        result.current.setFieldTouched('age')
      })

      // Then: both fields should be touched
      expect(result.current.formState.touched.name).toBe(true)
      expect(result.current.formState.touched.age).toBe(true)
    })

    test('should not trigger validation when marking touched', () => {
      // Given: hook with validation schema
      const { result } = renderHook(() =>
        useForm({
          schema: RequiredStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: marking field as touched
      act(() => {
        result.current.setFieldTouched('name')
      })

      // Then: errors should remain empty (validation not triggered)
      expect(result.current.formState.errors).toEqual([])
    })
  })

  describe('validateForm behavior', () => {
    test('should return success Effect for valid values', async () => {
      // Given: hook with valid values
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: 'John' },
        }),
      )

      // When: validating valid values
      const validationEffect = result.current.validateForm({ name: 'John' })
      const validationResult = await Effect.runPromise(
        validationEffect.pipe(Effect.either),
      )

      // Then: should succeed
      expect(Either.isRight(validationResult)).toBe(true)
      if (Either.isRight(validationResult)) {
        expect(validationResult.right).toEqual({ name: 'John' })
      }
    })

    test('should return failure Effect for invalid values', async () => {
      // Given: hook with validation schema
      const { result } = renderHook(() =>
        useForm({
          schema: RequiredStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: validating invalid values
      const validationEffect = result.current.validateForm({ name: '' })
      const validationResult = await Effect.runPromise(
        validationEffect.pipe(Effect.either),
      )

      // Then: should fail with errors
      expect(Either.isLeft(validationResult)).toBe(true)
      if (Either.isLeft(validationResult)) {
        const errors = validationResult.left
        expect(Array.isArray(errors)).toBe(true)
        expect(errors.length).toBeGreaterThan(0)
        expect(errors[0].field).toBe('name')
      }
    })

    test('should collect all validation errors', async () => {
      // Given: hook with multi-field schema
      const { result } = renderHook(() =>
        useForm({
          schema: MultiFieldSchema,
          initialValues: { name: '', age: -1 },
        }),
      )

      // When: validating invalid values for multiple fields
      const validationEffect = result.current.validateForm({
        name: '',
        age: -1,
      })
      const validationResult = await Effect.runPromise(
        validationEffect.pipe(Effect.either),
      )

      // Then: should return errors for all invalid fields
      expect(Either.isLeft(validationResult)).toBe(true)
      if (Either.isLeft(validationResult)) {
        const errors = validationResult.left
        expect(errors.length).toBeGreaterThanOrEqual(2)
        const fieldNames = errors.map((e) => e.field)
        expect(fieldNames).toContain('name')
        expect(fieldNames).toContain('age')
      }
    })

    test('should validate email format correctly', async () => {
      // Given: hook with email schema
      const { result } = renderHook(() =>
        useForm({
          schema: EmailSchema,
          initialValues: { email: '' },
        }),
      )

      // When: validating invalid email
      const invalidEffect = result.current.validateForm({ email: 'invalid' })
      const invalidResult = await Effect.runPromise(
        invalidEffect.pipe(Effect.either),
      )

      // Then: should fail
      expect(Either.isLeft(invalidResult)).toBe(true)

      // When: validating valid email
      const validEffect = result.current.validateForm({
        email: 'test@example.com',
      })
      const validResult = await Effect.runPromise(
        validEffect.pipe(Effect.either),
      )

      // Then: should succeed
      expect(Either.isRight(validResult)).toBe(true)
    })
  })

  describe('getFieldError behavior', () => {
    test('should return undefined when field has no error', () => {
      // Given: hook with no errors
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: getting error for field
      const error = result.current.getFieldError('name')

      // Then: should return undefined
      expect(error).toBeUndefined()
    })

    test('should return error message when field has error', async () => {
      // Given: hook with validation errors set
      const { result } = renderHook(() =>
        useForm({
          schema: RequiredStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: validating and setting errors
      const validationEffect = result.current.validateForm({ name: '' })
      const validationResult = await Effect.runPromise(
        validationEffect.pipe(Effect.either),
      )

      if (Either.isLeft(validationResult)) {
        act(() => {
          result.current.setValidationErrors(validationResult.left)
        })

        // Then: should return error message
        const error = result.current.getFieldError('name')
        expect(error).toBeDefined()
        expect(typeof error).toBe('string')
        expect(error).toContain('required')
      }
    })
  })

  describe('setValidationErrors behavior', () => {
    test('should update form state with errors', () => {
      // Given: initialized hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: setting validation errors
      const errors: FieldError[] = [
        { field: 'name', message: 'Name is required' },
      ]
      act(() => {
        result.current.setValidationErrors(errors)
      })

      // Then: errors should be set
      expect(result.current.formState.errors).toEqual(errors)
      expect(result.current.formState.isValid).toBe(false)
    })

    test('should mark fields with errors as touched', () => {
      // Given: initialized hook
      const { result } = renderHook(() =>
        useForm({
          schema: MultiFieldSchema,
          initialValues: { name: '', age: 0 },
        }),
      )

      // When: setting validation errors
      const errors: FieldError[] = [
        { field: 'name', message: 'Name is required' },
        { field: 'age', message: 'Age must be non-negative' },
      ]
      act(() => {
        result.current.setValidationErrors(errors)
      })

      // Then: fields with errors should be marked as touched
      expect(result.current.formState.touched.name).toBe(true)
      expect(result.current.formState.touched.age).toBe(true)
    })

    test('should set isValid to true when no errors', () => {
      // Given: hook with errors
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: setting empty errors array
      act(() => {
        result.current.setValidationErrors([])
      })

      // Then: isValid should be true
      expect(result.current.formState.isValid).toBe(true)
    })

    test('should set isValid to false when errors exist', () => {
      // Given: initialized hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: setting errors
      const errors: FieldError[] = [
        { field: 'name', message: 'Name is required' },
      ]
      act(() => {
        result.current.setValidationErrors(errors)
      })

      // Then: isValid should be false
      expect(result.current.formState.isValid).toBe(false)
    })

    test('should not mark root errors as touched', () => {
      // Given: initialized hook
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues: { name: '' },
        }),
      )

      // When: setting root error
      const errors: FieldError[] = [{ field: 'root', message: 'Form error' }]
      act(() => {
        result.current.setValidationErrors(errors)
      })

      // Then: root should not be in touched
      expect(result.current.formState.touched.root).toBeUndefined()
    })
  })

  describe('resetForm behavior', () => {
    test('should reset form to initial state', () => {
      // Given: hook with modified state
      const initialValues = { name: 'John' }
      const { result } = renderHook(() =>
        useForm({
          schema: SimpleStringSchema,
          initialValues,
        }),
      )

      // When: modifying state then resetting
      act(() => {
        result.current.setFieldValue('name', 'Jane')
        result.current.setFieldTouched('name')
        result.current.setValidationErrors([
          { field: 'name', message: 'Error' },
        ])
      })

      act(() => {
        result.current.resetForm()
      })

      // Then: should return to initial state
      expect(result.current.formState.values).toEqual(initialValues)
      expect(result.current.formState.errors).toEqual([])
      expect(result.current.formState.isValid).toBe(false)
      expect(result.current.formState.touched).toEqual({})
    })
  })

  describe('integration behavior', () => {
    test('should work with complex validation scenarios', async () => {
      // Given: hook with email validation
      const { result } = renderHook(() =>
        useForm({
          schema: EmailSchema,
          initialValues: { email: '' },
        }),
      )

      // When: setting invalid email and validating
      act(() => {
        result.current.setFieldValue('email', 'invalid-email')
      })

      const validationEffect = result.current.validateForm({
        email: 'invalid-email',
      })
      const validationResult = await Effect.runPromise(
        validationEffect.pipe(Effect.either),
      )

      if (Either.isLeft(validationResult)) {
        act(() => {
          result.current.setValidationErrors(validationResult.left)
        })

        // Then: should have error
        const error = result.current.getFieldError('email')
        expect(error).toBeDefined()
        expect(error).toContain('email')
      }
    })

    test('should maintain state consistency across multiple operations', () => {
      // Given: initialized hook
      const { result } = renderHook(() =>
        useForm({
          schema: MultiFieldSchema,
          initialValues: { name: '', age: 0 },
        }),
      )

      // When: performing multiple operations
      act(() => {
        result.current.setFieldValue('name', 'John')
        result.current.setFieldValue('age', 25)
        result.current.setFieldTouched('name')
        result.current.setFieldTouched('age')
      })

      // Then: state should be consistent
      expect(result.current.formState.values.name).toBe('John')
      expect(result.current.formState.values.age).toBe(25)
      expect(result.current.formState.touched.name).toBe(true)
      expect(result.current.formState.touched.age).toBe(true)
    })
  })
})

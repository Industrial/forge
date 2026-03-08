import { Schema, Effect, Either, ParseResult } from 'effect'
import { useState, useCallback } from 'react'

import { RegisterFormSchema } from '@/features/authentication/schemas/RegisterFormSchema'

export interface FieldError {
  readonly field: string
  readonly message: string
}

export interface FormState {
  readonly values: {
    readonly email: string
    readonly password: string
  }
  readonly errors: readonly FieldError[]
  readonly isValid: boolean
  readonly touched: {
    readonly email: boolean
    readonly password: boolean
  }
}

const initialFormState: FormState = {
  values: {
    email: '',
    password: '',
  },
  errors: [],
  isValid: false,
  touched: {
    email: false,
    password: false,
  },
}

/**
 * Effect.ts-based form hook for registration form.
 * Handles validation using Effect.ts Schema and provides Effect-based handlers.
 */
export function useRegisterForm() {
  const [formState, setFormState] = useState<FormState>(initialFormState)

  /**
   * Extracts all field errors from a Schema.ParseError using Effect.ts's ArrayFormatter.
   * The formatter handles nested/composite errors automatically.
   */
  const extractFieldErrors = (
    parseError: ParseResult.ParseError,
  ): FieldError[] => {
    const issues = ParseResult.ArrayFormatter.formatErrorSync(parseError)
    return issues.map((issue) => ({
      field: issue.path.length > 0 ? String(issue.path[0]) : 'root',
      message: issue.message,
    }))
  }

  /**
   * Validates form values using Effect.ts Schema.
   * Returns an Effect that succeeds with validated data or fails with validation errors.
   * Uses { errors: "all" } to collect all validation errors, not just the first one.
   */
  const validateForm = useCallback(
    (values: FormState['values']) => {
      const result = Schema.decodeUnknownEither(RegisterFormSchema, {
        errors: 'all',
      })(values)

      if (Either.isLeft(result)) {
        return Effect.fail(extractFieldErrors(result.left))
      }

      return Effect.succeed(result.right)
    },
    [],
  )

  /**
   * Updates a field value without validation.
   * Validation only happens when the form is submitted.
   */
  const setFieldValue = useCallback((field: 'email' | 'password', value: string) => {
    setFormState((prev) => ({
      ...prev,
      values: { ...prev.values, [field]: value },
    }))
  }, [])

  /**
   * Marks a field as touched without validation.
   * Validation only happens when the form is submitted.
   */
  const setFieldTouched = useCallback((field: 'email' | 'password') => {
    setFormState((prev) => ({
      ...prev,
      touched: { ...prev.touched, [field]: true },
    }))
  }, [])

  /**
   * Gets error message for a specific field.
   */
  const getFieldError = useCallback(
    (field: string): string | undefined => {
      const error = formState.errors.find((e) => e.field === field)
      return error?.message
    },
    [formState.errors],
  )

  /**
   * Updates form state with validation errors (used on submit).
   */
  const setValidationErrors = useCallback((errors: readonly FieldError[]) => {
    setFormState((prev) => ({
      ...prev,
      errors,
      isValid: errors.length === 0,
      touched: {
        email: true,
        password: true,
      },
    }))
  }, [])

  /**
   * Resets the form to initial state.
   */
  const resetForm = useCallback(() => {
    setFormState(initialFormState)
  }, [])

  return {
    formState,
    setFieldValue,
    setFieldTouched,
    getFieldError,
    validateForm,
    setValidationErrors,
    resetForm,
  }
}

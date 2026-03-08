import { useState, useCallback } from 'react'
import { Schema, Effect, Either, pipe, ParseResult } from 'effect'

import { RegisterFormSchema } from '@/features/authentication/schemas/RegisterFormSchema'
import type { ParseError } from 'effect/ParseResult'

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
  const extractFieldErrors = (parseError: ParseError): FieldError[] => {
    const issues = ParseResult.ArrayFormatter.formatErrorSync(parseError)
    debugger
    const result = issues.map((issue) => ({
      field: issue.path.length > 0 ? String(issue.path[0]) : 'root',
      message: issue.message,
    }))
    debugger
    return result
  }

  /**
   * Validates form values using Effect.ts Schema.
   * Returns an Effect that succeeds with validated data or fails with validation errors.
   */
  const validateForm = useCallback(
    (values: FormState['values']) =>
      pipe(
        Schema.decodeUnknownEither(RegisterFormSchema)(values),
        Either.match({
          onLeft: (parseError: ParseError) =>
            Effect.fail(extractFieldErrors(parseError)),
          onRight: (validated) => Effect.succeed(validated),
        }),
      ),
    [],
  )

  /**
   * Updates a field value and validates if the field has been touched.
   */
  const setFieldValue = useCallback(
    (field: 'email' | 'password', value: string) => {
      setFormState((prev) => {
        const newValues = { ...prev.values, [field]: value }
        const isTouched = prev.touched[field]

        // Only validate if field has been touched
        if (isTouched) {
          const validationEffect = validateForm(newValues)
          const result = Effect.runSync(
            validationEffect.pipe(
              Effect.either,
              Effect.map((either) =>
                Either.match(either, {
                  onLeft: (errors) => ({ errors, isValid: false }),
                  onRight: () => ({ errors: [], isValid: true }),
                }),
              ),
            ),
          )

          return {
            ...prev,
            values: newValues,
            errors: result.errors,
            isValid: result.isValid,
          }
        }

        return {
          ...prev,
          values: newValues,
        }
      })
    },
    [validateForm],
  )

  /**
   * Marks a field as touched.
   */
  const setFieldTouched = useCallback(
    (field: 'email' | 'password') => {
      setFormState((prev) => {
        const newTouched = { ...prev.touched, [field]: true }
        const newValues = prev.values

        // Validate when field is touched
        const validationEffect = validateForm(newValues)
        const result = Effect.runSync(
          validationEffect.pipe(
            Effect.either,
            Effect.map((either) =>
              Either.match(either, {
                onLeft: (errors) => ({ errors, isValid: false }),
                onRight: () => ({ errors: [], isValid: true }),
              }),
            ),
          ),
        )

        return {
          ...prev,
          touched: newTouched,
          errors: result.errors,
          isValid: result.isValid,
        }
      })
    },
    [validateForm],
  )

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

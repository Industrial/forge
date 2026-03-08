import { Schema, Effect, Either, ParseResult } from 'effect'
import { useState, useCallback } from 'react'

export interface FieldError {
  readonly field: string
  readonly message: string
}

/**
 * Generalized form state that works with any schema.
 * The values type is inferred from the schema.
 */
export interface FormState<TValues extends Record<string, unknown>> {
  readonly values: TValues
  readonly errors: readonly FieldError[]
  readonly isValid: boolean
  readonly touched: Partial<Record<keyof TValues, boolean>>
}

/**
 * Configuration for useForm hook.
 * Type is inferred from the schema.
 */
export interface UseFormConfig<TSchema extends Schema.Schema<any, any, never>> {
  /**
   * Effect.ts Schema for form validation.
   * Must have Context = never (no dependencies).
   * The form values type is inferred from this schema.
   */
  readonly schema: TSchema
  /**
   * Initial form values matching the schema type.
   */
  readonly initialValues: Schema.Schema.Type<TSchema>
}

/**
 * Generalized Effect.ts-based form hook.
 * Handles validation using Effect.ts Schema and provides Effect-based handlers.
 * The form values type is automatically inferred from the schema.
 *
 * @template TSchema - The Effect.ts Schema type (inferred from schema parameter)
 *
 * @example
 * ```tsx
 * const { formState, setFieldValue, validateForm } = useForm({
 *   schema: MyFormSchema,
 *   initialValues: { email: '', password: '' }
 * })
 * ```
 */
export function useForm<TSchema extends Schema.Schema<any, any, never>>(
  config: UseFormConfig<TSchema>,
) {
  type TValues = Schema.Schema.Type<TSchema>
  const initialFormState: FormState<TValues> = {
    values: config.initialValues,
    errors: [],
    isValid: false,
    touched: {},
  }

  const [formState, setFormState] =
    useState<FormState<TValues>>(initialFormState)

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
    (values: TValues) => {
      const result = Schema.decodeUnknownEither(config.schema, {
        errors: 'all',
      })(values)

      if (Either.isLeft(result)) {
        return Effect.fail(extractFieldErrors(result.left))
      }

      return Effect.succeed(result.right)
    },
    [config.schema],
  )

  /**
   * Updates a field value without validation.
   * Validation only happens when the form is submitted.
   */
  const setFieldValue = useCallback(
    <K extends keyof TValues>(field: K, value: TValues[K]) => {
      setFormState((prev) => ({
        ...prev,
        values: { ...prev.values, [field]: value },
      }))
    },
    [],
  )

  /**
   * Marks a field as touched without validation.
   * Validation only happens when the form is submitted.
   */
  const setFieldTouched = useCallback(<K extends keyof TValues>(field: K) => {
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
    setFormState((prev) => {
      // Mark all fields that have errors as touched
      const touchedFields = errors.reduce(
        (acc, error) => {
          if (error.field !== 'root') {
            acc[error.field as keyof TValues] = true
          }
          return acc
        },
        { ...prev.touched } as Partial<Record<keyof TValues, boolean>>,
      )

      return {
        ...prev,
        errors,
        isValid: errors.length === 0,
        touched: touchedFields,
      }
    })
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

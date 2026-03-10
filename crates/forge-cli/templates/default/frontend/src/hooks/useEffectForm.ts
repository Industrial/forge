import { Effect, Either, type Schema } from 'effect'
import { useCallback, useState } from 'react'
import type { FieldError } from './useForm'
import { useForm } from './useForm'
import type { UseFormConfig } from './useForm'

/**
 * Result of a submit effect. The effect should never reject; use catchAll to return one of these.
 */
export type SubmitResult =
  | { type: 'success' }
  | { type: 'validation'; errors: readonly FieldError[] }
  | { type: 'error'; message: string }

/**
 * Effect-based form hook that adds submit state and a handler to run an Effect on submit.
 * Validates synchronously first; if valid, runs the effect (which should return a SubmitResult).
 * Use for login, register, and other forms that submit via Effect and need submitting/error state.
 */
export function useEffectForm<TSchema extends Schema.Schema<any, any, never>>(
  config: UseFormConfig<TSchema>,
) {
  type TValues = Schema.Schema.Type<TSchema>
  const form = useForm(config)
  const [submitting, setSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)

  const clearSubmitError = useCallback(() => {
    setSubmitError(null)
  }, [])

  /**
   * Returns a submit handler that validates then runs the given effect.
   * The effect should return a SubmitResult (use Effect.catchAll to map failures to result).
   */
  const handleSubmitEffect = useCallback(
    (
      buildEffect: (values: TValues) => Effect.Effect<SubmitResult>,
    ): ((e: React.FormEvent<HTMLFormElement>) => void) => {
      return (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault()
        const currentValues = form.formState.values as TValues
        const validationEffect = form.validateForm(currentValues)
        const validationResult = Effect.runSync(
          validationEffect.pipe(
            Effect.either,
            Effect.map((either) =>
              Either.match(either, {
                onLeft: (errors) => ({ errors, isValid: false as const }),
                onRight: () => ({
                  errors: [] as readonly FieldError[],
                  isValid: true as const,
                }),
              }),
            ),
          ),
        )
        form.setValidationErrors(validationResult.errors)
        if (!validationResult.isValid) {
          return
        }

        setSubmitting(true)
        setSubmitError(null)
        Effect.runPromise(buildEffect(currentValues))
          .then((result) => {
            setSubmitting(false)
            if (result.type === 'success') {
              return
            }
            if (result.type === 'validation') {
              form.setValidationErrors(result.errors)
            } else {
              setSubmitError(result.message)
            }
          })
          .catch(() => {
            setSubmitting(false)
            setSubmitError('Something went wrong')
          })
      }
    },
    [form.formState.values, form.validateForm, form.setValidationErrors],
  )

  return {
    ...form,
    submitting,
    submitError,
    clearSubmitError,
    setSubmitError,
    handleSubmitEffect,
  }
}

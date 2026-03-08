import { Schema } from 'effect'

/**
 * Required validation message: custom error message for required fields.
 */
export const requiredMessage = () => 'This field is required.' as const

/**
 * Required filter transformation: validates that a string is not empty.
 * Provides custom error message "This field is required."
 */
export const requiredTransform = Schema.filter((s: string) => s.length > 0, {
  message: requiredMessage,
})

/**
 * Required string schema: validates that a string is not empty or whitespace-only.
 * Provides custom error message "This field is required."
 * Convenience export for Schema.String with required validation.
 */
export const RequiredStringSchema = Schema.String.pipe(requiredTransform)

export type RequiredString = Schema.Schema.Type<typeof RequiredStringSchema>

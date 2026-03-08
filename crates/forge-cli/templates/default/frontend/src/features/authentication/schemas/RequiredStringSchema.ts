import { Schema } from 'effect'

/**
 * Required string schema: validates that a string is not empty or whitespace-only.
 * Provides custom error message "This field is required."
 * Automatically trims whitespace from the input.
 *
 * Uses Schema.NonEmptyTrimmedString as the base (which handles trimming and non-empty validation),
 * then applies a filter with a custom error message to override the default message.
 */
export const RequiredStringSchema = Schema.String.pipe(
  Schema.filter((s: string) => s.length > 0, {
    message: () => 'This field is required.',
  }),
)

export type RequiredString = Schema.Schema.Type<typeof RequiredStringSchema>

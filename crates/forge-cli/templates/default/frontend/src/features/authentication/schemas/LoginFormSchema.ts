import { Schema } from 'effect'

import { requiredMessage } from '@/schemas/RequiredStringSchema'
import { emailPattern, emailMessage } from '@/schemas/EmailSchema'

/**
 * Login form schema combining required validation with field-specific validation.
 * Required validation comes first (with custom error message), then field-specific validation is applied.
 *
 * Transformations are inlined to ensure TypeScript correctly infers Context = never.
 *
 * Note: Login form only requires email validation (not password length validation like registration).
 */
export const LoginFormSchema = Schema.Struct({
  email: Schema.String.pipe(
    Schema.filter((s: string) => s.length > 0, {
      message: requiredMessage,
    }),
  ).pipe(
    Schema.pattern(emailPattern, {
      message: emailMessage,
    }),
  ),
  password: Schema.String.pipe(
    Schema.filter((s: string) => s.length > 0, {
      message: requiredMessage,
    }),
  ),
})

export type LoginFormValues = Schema.Schema.Type<typeof LoginFormSchema>

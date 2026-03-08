import { Schema } from 'effect'

import { RequiredStringSchema } from '@/features/authentication/schemas/RequiredStringSchema'
import { EmailSchema } from '@/features/authentication/schemas/EmailSchema'
import { PasswordSchema } from '@/features/authentication/schemas/PasswordSchema'

/**
 * Registration form schema combining required validation with field-specific validation.
 * Required validation comes first (with custom error message), then field-specific validation is applied.
 */
export const RegisterFormSchema = Schema.Struct({
  email: RequiredStringSchema.pipe(EmailSchema),
  password: RequiredStringSchema.pipe(PasswordSchema),
})

export type RegisterFormValues = Schema.Schema.Type<typeof RegisterFormSchema>

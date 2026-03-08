import { Schema } from 'effect'

/**
 * Password validation options: minimum length and user-friendly error message.
 */
const passwordValidationOptions = {
  message: () => 'Password must be at least 8 characters' as const,
}

/**
 * Applies password length validation to any string schema.
 * Does NOT validate that the field is required - that should be handled at the form schema level.
 *
 * @example
 * ```ts
 * const schema = applyPasswordValidation(Schema.String)
 * const requiredPassword = applyPasswordValidation(RequiredStringSchema)
 * ```
 */
export const applyPasswordValidation = (schema: Schema.Schema<string>) =>
  schema.pipe(
    Schema.minLength(8, passwordValidationOptions),
  ) as Schema.Schema<string>

/**
 * Password schema: validates minimum length of 8 characters.
 * Uses Schema.String as the base - for required validation, use applyPasswordValidation(RequiredStringSchema).
 * Provides user-friendly error message for short passwords.
 */
export const PasswordSchema = applyPasswordValidation(Schema.String)

export type Password = Schema.Schema.Type<typeof PasswordSchema>

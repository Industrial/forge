import { Schema } from 'effect'

/**
 * Password minimum length: 8 characters.
 */
export const passwordMinLength = 8

/**
 * Password validation message: user-friendly error message for short passwords.
 */
export const passwordMessage = () =>
  'Password must be at least 8 characters' as const

/**
 * Password length transformation: applies minimum length validation to any string schema.
 */
export const passwordLengthTransform = Schema.minLength(passwordMinLength, {
  message: passwordMessage,
})

/**
 * Password schema: validates minimum length of 8 characters.
 * Convenience export for Schema.String with password length validation.
 */
export const PasswordSchema = Schema.String.pipe(passwordLengthTransform)

export type Password = Schema.Schema.Type<typeof PasswordSchema>

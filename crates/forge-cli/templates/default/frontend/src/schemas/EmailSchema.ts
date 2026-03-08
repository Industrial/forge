import { Schema } from 'effect'

/**
 * Email pattern regex: RFC 5322 compliant email pattern.
 */
export const emailPattern =
  /^(?=.{1,254}$)(?=.{1,64}@)[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+)*@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/

/**
 * Email validation message: user-friendly error message for invalid email formats.
 */
export const emailMessage = () => 'Enter a valid email address' as const

/**
 * Email pattern transformation: applies email pattern validation to any string schema.
 */
export const emailPatternTransform = Schema.pattern(emailPattern, {
  message: emailMessage,
})

/**
 * Email schema: validates email format using RFC 5322 compliant regex pattern.
 * Convenience export for Schema.String with email validation.
 */
export const EmailSchema = Schema.String.pipe(emailPatternTransform)

export type Email = Schema.Schema.Type<typeof EmailSchema>

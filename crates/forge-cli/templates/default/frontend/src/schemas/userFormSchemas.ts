import { Schema } from 'effect'
import {
  emailSchema,
  orgIdSchema,
  passwordMin8Schema,
  roleIdsSchema,
} from './fragments'

export const loginFormSchema = Schema.Struct({
  email: emailSchema,
  password: Schema.String,
})

export type LoginFormValues = Schema.Schema.Type<typeof loginFormSchema>

export const registerFormSchema = Schema.Struct({
  email: emailSchema,
  password: passwordMin8Schema,
})

export type RegisterFormValues = Schema.Schema.Type<typeof registerFormSchema>

export const userAddFormSchema = Schema.Struct({
  email: emailSchema,
  password: passwordMin8Schema,
  orgId: orgIdSchema,
  roleIds: roleIdsSchema,
})

export type UserAddFormValues = Schema.Schema.Type<typeof userAddFormSchema>

const roleIdsNonEmpty = roleIdsSchema.pipe(
  Schema.filter((arr) => arr.length >= 1, {
    message: () => 'Select at least one role',
  }),
)

export const userAddFormSchemaStrict = Schema.Struct({
  email: emailSchema,
  password: passwordMin8Schema,
  orgId: orgIdSchema,
  roleIds: roleIdsNonEmpty,
})

export type UserAddFormValuesStrict = Schema.Schema.Type<
  typeof userAddFormSchemaStrict
>

export const userEditFormSchema = Schema.Struct({
  email: Schema.NonEmptyTrimmedString,
  active: Schema.Boolean,
})

export type UserEditFormValues = Schema.Schema.Type<typeof userEditFormSchema>

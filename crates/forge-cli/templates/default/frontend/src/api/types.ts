/**
 * Shared API types for REST and RPC. Same shapes as backend (forge-query).
 * Schemas are the single source of truth; types are derived via Schema.Schema.Type.
 */

import { Schema } from 'effect'

// ---------------------------------------------------------------------------
// List / query (REST and RPC use the same params and list shape)
// ---------------------------------------------------------------------------

export const ListQueryParamsSchema = Schema.Struct({
  filter: Schema.optional(Schema.String),
  sort: Schema.optional(Schema.String),
  order: Schema.optional(Schema.String),
  offset: Schema.optional(Schema.Number),
  limit: Schema.optional(Schema.Number),
})

export type ListQueryParams = Schema.Schema.Type<typeof ListQueryParamsSchema>

/** List response: { data: T[] }. Type mirrors ListResponseSchema output. */
export type ListResponse<T = unknown> = { readonly data: readonly T[] }

export const ListResponseSchema = <A, I = unknown>(
  item: Schema.Schema<A, I>,
): Schema.Schema<ListResponse<A>, { data: readonly I[] }> =>
  Schema.Struct({
    data: Schema.Array(item),
  })

/** Single item response (e.g. GET /api/entities/:id/:id). Type mirrors GetResponseSchema output. */
export type GetResponse<T = unknown> = { readonly data: T }

export const GetResponseSchema = <A, I = unknown>(
  item: Schema.Schema<A, I>,
): Schema.Schema<GetResponse<A>, { data: I }> =>
  Schema.Struct({
    data: item,
  })

// ---------------------------------------------------------------------------
// RPC envelope (POST /api/rpc)
// ---------------------------------------------------------------------------

export const RpcSubscribeRequestSchema = Schema.Struct({
  method: Schema.Literal('subscribe'),
  entity_id: Schema.String,
  params: Schema.optional(ListQueryParamsSchema),
})

export type RpcSubscribeRequest = Schema.Schema.Type<
  typeof RpcSubscribeRequestSchema
>

export const RpcSubscribeResultSchema = Schema.Struct({
  subscription_id: Schema.String,
})

export type RpcSubscribeResult = Schema.Schema.Type<
  typeof RpcSubscribeResultSchema
>

/** RPC response envelope. Type mirrors RpcResponseSchema output. */
export type RpcResponse<T> = {
  readonly result?: T
  readonly error?: { readonly message?: string }
}

const RpcErrorSchema = Schema.Struct({
  message: Schema.optional(Schema.String),
})

export const RpcResponseSchema = <A, I = unknown>(
  result: Schema.Schema<A, I>,
): Schema.Schema<RpcResponse<A>, RpcResponse<I>> =>
  Schema.Struct({
    result: Schema.optional(result),
    error: Schema.optional(RpcErrorSchema),
  })

// ---------------------------------------------------------------------------
// Auth: /api/auth/me, /api/auth/login, /api/auth/register
// ---------------------------------------------------------------------------

const AuthMeUserSchema = Schema.Struct({
  id: Schema.optional(Schema.String),
  email: Schema.optional(Schema.String),
  permissions: Schema.optional(Schema.Array(Schema.String)),
})

export const AuthMeBodySchema = Schema.Struct({
  user: Schema.optional(AuthMeUserSchema),
  needs_scope_select: Schema.optional(Schema.Boolean),
  permissions: Schema.optional(Schema.Array(Schema.String)),
  profiles: Schema.optional(Schema.Array(Schema.Unknown)),
  flash: Schema.optional(Schema.Unknown),
})

export type AuthMeBody = Schema.Schema.Type<typeof AuthMeBodySchema>

export const LoginResponseSchema = Schema.Struct({
  ok: Schema.Boolean,
  token: Schema.String,
  needs_profile_select: Schema.optional(Schema.Boolean),
})

export type LoginResponse = Schema.Schema.Type<typeof LoginResponseSchema>

export const RegisterResponseSchema = Schema.Struct({
  ok: Schema.optional(Schema.Boolean),
  message: Schema.optional(Schema.String),
})

export type RegisterResponse = Schema.Schema.Type<typeof RegisterResponseSchema>

// ---------------------------------------------------------------------------
// API error (4xx/5xx)
// ---------------------------------------------------------------------------

const ApiErrorDetailSchema = Schema.Union(
  Schema.String,
  Schema.Struct({ message: Schema.optional(Schema.String) }),
)

export const ApiErrorBodySchema = Schema.Struct({
  message: Schema.optional(Schema.String),
  error: Schema.optional(ApiErrorDetailSchema),
})

export type ApiErrorBody = Schema.Schema.Type<typeof ApiErrorBodySchema>

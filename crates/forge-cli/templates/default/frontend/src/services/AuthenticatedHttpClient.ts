/**
 * Tag for the authenticated HTTP client (adds token + scope headers).
 * Used by EntityApiLive and RpcApiLive. Same interface as HttpClient.
 */
import type { HttpClient } from '@effect/platform'
import { Context } from 'effect'

export const AuthenticatedHttpClient =
  Context.GenericTag<HttpClient.HttpClient>('@forge/AuthenticatedHttpClient')

/**
 * AuditLog service – backend operations for the audit log page.
 *
 * Provides list with pagination and filters. All methods return
 * `Effect<A, Error, never>`; Live implementation uses HttpClient.
 */

import { Context, type Effect } from 'effect'
import type { AuditLogEntry } from '../domain/AuditLogEntry'

export interface AuditLogListParams {
  readonly limit: number
  readonly offset: number
  readonly from?: string
  readonly to?: string
  readonly outcome?: string
  readonly event_kind?: string
  readonly action?: string
  readonly reason?: string
}

export interface AuditLogResult {
  readonly entries: readonly AuditLogEntry[]
  readonly total: number
}

export interface AuditLogService {
  /** List audit log entries with filters and pagination. */
  readonly list: (
    params: AuditLogListParams,
  ) => Effect.Effect<AuditLogResult, Error, never>
}

export const AuditLog =
  Context.GenericTag<AuditLogService>('dashboard/AuditLog')

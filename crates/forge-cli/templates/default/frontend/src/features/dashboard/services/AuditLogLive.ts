/**
 * Live implementation of AuditLog service using HttpClient.
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { AuditLog } from './AuditLog'
import { AuditLogEntry } from '../domain/AuditLogEntry'
import type { AuditLogListParams, AuditLogResult, AuditLogService } from './AuditLog'

function parseErr(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

const AuditLogLive = Layer.effect(
  AuditLog,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const list: AuditLogService['list'] = (params: AuditLogListParams) =>
      Effect.gen(function* () {
        const search = new URLSearchParams()
        search.set('limit', String(params.limit))
        search.set('offset', String(params.offset))
        if (params.from) search.set('from', params.from)
        if (params.to) search.set('to', params.to)
        if (params.outcome) search.set('outcome', params.outcome)
        if (params.event_kind) search.set('event_kind', params.event_kind)
        if (params.action) search.set('action', params.action)
        if (params.reason?.trim()) search.set('reason', params.reason.trim())
        const url = `/api/dashboard/audit-log?${search.toString()}`
        const response = yield* client.execute(HttpClientRequest.get(url))
        const body = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to view the audit log.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseErr(body)))
        }
        const data = body as {
          entries?: Array<{
            id: string
            event_kind: string
            actor_id: string
            subject_id?: string | null
            organization_id?: string | null
            action: string
            resource_type: string
            resource_id?: string | null
            outcome: string
            reason?: string | null
            occurred_at: string
          }>
          total?: number
        }
        const raw = data.entries ?? []
        const entries = raw.map(
          (e) =>
            new AuditLogEntry({
              id: e.id,
              event_kind: e.event_kind,
              actor_id: e.actor_id,
              subject_id: e.subject_id,
              organization_id: e.organization_id,
              action: e.action,
              resource_type: e.resource_type,
              resource_id: e.resource_id,
              outcome: e.outcome,
              reason: e.reason,
              occurred_at: e.occurred_at,
            }),
        ) as readonly AuditLogEntry[]
        const result: AuditLogResult = {
          entries,
          total: data.total ?? 0,
        }
        return result
      })

    return { list }
  }),
)

export { AuditLogLive }

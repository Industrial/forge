import { Data } from 'effect'

/**
 * Audit log entry. Used by the AuditLog service.
 */
export class AuditLogEntry extends Data.TaggedClass('AuditLogEntry')<{
  readonly id: string
  readonly event_kind: string
  readonly actor_id: string
  readonly subject_id?: string | null
  readonly organization_id?: string | null
  readonly action: string
  readonly resource_type: string
  readonly resource_id?: string | null
  readonly outcome: string
  readonly reason?: string | null
  readonly occurred_at: string
}> {}

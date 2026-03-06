/**
 * Mock AuditLog service for tests.
 */

import { Effect, Layer } from 'effect'
import { AuditLog } from './AuditLog'
import { AuditLogEntry } from '../domain/AuditLogEntry'
import type {
  AuditLogListParams,
  AuditLogResult,
  AuditLogService,
} from './AuditLog'

/**
 * Creates a mock AuditLog service. Optionally pass initial entries.
 */
export function createAuditLogMock(
  initialEntries: readonly AuditLogEntry[] = [],
): AuditLogService {
  const entries: AuditLogEntry[] = initialEntries.map((e) =>
    e instanceof AuditLogEntry ? e : new AuditLogEntry(e),
  )

  return {
    list: (params: AuditLogListParams) =>
      Effect.sync((): AuditLogResult => {
        const start = params.offset
        const end = Math.min(params.offset + params.limit, entries.length)
        const page = entries.slice(start, end)
        return {
          entries: page as readonly AuditLogEntry[],
          total: entries.length,
        }
      }),
  }
}

export const AuditLogMockLayer = (
  initialEntries: readonly AuditLogEntry[] = [],
) => Layer.succeed(AuditLog, createAuditLogMock(initialEntries))

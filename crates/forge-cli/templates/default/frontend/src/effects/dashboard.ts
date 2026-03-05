import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect } from 'effect'

export type Organization = {
  id: string
  name: string
  slug: string
  created_at?: string
  updated_at?: string
}

export type DashboardRole = {
  id: string
  name: string
  display_name: string | null
}

/**
 * Fetches organizations from GET /api/dashboard/organizations.
 * Requires HttpClient.
 */
export const fetchOrganizationsEffect = Effect.gen(function* () {
  const client = yield* HttpClient.HttpClient
  const response = yield* client.execute(
    HttpClientRequest.get('/api/dashboard/organizations'),
  )
  const body = yield* response.json
  if (response.status < 200 || response.status >= 300) {
    const msg =
      typeof body === 'object' && body !== null && 'error' in body
        ? String((body as { error: unknown }).error)
        : `HTTP ${response.status}`
    return yield* Effect.fail(new Error(msg))
  }
  const data = body as { organizations?: Organization[] }
  return (data.organizations ?? []) as Organization[]
})

/**
 * Fetches roles for an org from GET /api/dashboard/roles?org_id=...
 * Requires HttpClient.
 */
export const fetchRolesEffect = (orgId: string) =>
  Effect.gen(function* () {
    if (!orgId) return [] as DashboardRole[]
    const client = yield* HttpClient.HttpClient
    const url = `/api/dashboard/roles?org_id=${encodeURIComponent(orgId)}`
    const response = yield* client.execute(HttpClientRequest.get(url))
    const body = yield* response.json
    if (response.status < 200 || response.status >= 300) {
      return [] as DashboardRole[]
    }
    const data = body as { roles?: DashboardRole[] }
    return (data.roles ?? []) as DashboardRole[]
  })

function parseErr(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

/** Create organization. Requires HttpClient. */
export const createOrganizationEffect = (body: {
  name: string
  slug?: string
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.post('/api/dashboard/organizations').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to create organizations.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

/** Update organization. Requires HttpClient. */
export const updateOrganizationEffect = (body: {
  id: string
  name?: string
  slug?: string
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.patch('/api/dashboard/organizations').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to update organizations.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

/** Delete organization. Requires HttpClient. */
export const deleteOrganizationEffect = (id: string) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.del('/api/dashboard/organizations').pipe(
      HttpClientRequest.bodyUnsafeJson({ id }),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to delete organizations.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

export type Role = {
  id: string
  org_id: string
  name: string
  display_name: string | null
  created_at?: string
  updated_at?: string
}

/** Fetch all roles (GET /api/dashboard/roles). Requires HttpClient. */
export const fetchRolesListEffect = Effect.gen(function* () {
  const client = yield* HttpClient.HttpClient
  const response = yield* client.execute(
    HttpClientRequest.get('/api/dashboard/roles'),
  )
  const body = yield* response.json
  if (response.status === 403) {
    return yield* Effect.fail(
      new Error('You do not have permission to view roles.'),
    )
  }
  if (response.status < 200 || response.status >= 300) {
    return yield* Effect.fail(new Error(parseErr(body)))
  }
  const data = body as { roles?: Role[] }
  return (data.roles ?? []) as Role[]
})

/** Create role. Requires HttpClient. */
export const createRoleEffect = (body: {
  org_id: string
  name: string
  display_name?: string
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.post('/api/dashboard/roles').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to create roles.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

/** Update role. Requires HttpClient. */
export const updateRoleEffect = (body: {
  id: string
  name?: string
  display_name?: string
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.patch('/api/dashboard/roles').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to update roles.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

/** Delete role. Requires HttpClient. */
export const deleteRoleEffect = (id: string) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.del('/api/dashboard/roles').pipe(
      HttpClientRequest.bodyUnsafeJson({ id }),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to delete roles.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

export type Assignment = {
  scope: string
  role_name: string
  permission_key: string
  org_id?: string | null
}

/** Fetch role-permissions assignments and permissions list. Requires HttpClient. */
export const fetchPermissionsDataEffect = Effect.gen(function* () {
  const client = yield* HttpClient.HttpClient
  const [assignRes, permRes] = yield* Effect.all([
    client.execute(HttpClientRequest.get('/api/dashboard/role-permissions')),
    client.execute(HttpClientRequest.get('/api/dashboard/permissions')),
  ])
  const assignBody = yield* assignRes.json
  const permBody = yield* permRes.json
  if (assignRes.status === 403 || permRes.status === 403) {
    return yield* Effect.fail(
      new Error('You do not have permission to manage permissions.'),
    )
  }
  if (
    assignRes.status >= 200 &&
    assignRes.status < 300 &&
    permRes.status >= 200 &&
    permRes.status < 300
  ) {
    const assignments =
      (assignBody as { assignments?: Assignment[] }).assignments ?? []
    const permissions =
      (permBody as { permissions?: string[] }).permissions ?? []
    return { assignments, permissions } as {
      assignments: Assignment[]
      permissions: string[]
    }
  }
  return yield* Effect.fail(new Error('Failed to load data.'))
})

/** Add role-permission assignment. Requires HttpClient. */
export const addPermissionAssignmentEffect = (body: {
  scope: string
  role_name: string
  permission_key: string
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.post('/api/dashboard/role-permissions').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to manage permissions.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

/** Remove role-permission assignment. Requires HttpClient. */
export const deletePermissionAssignmentEffect = (body: {
  scope: string
  role_name: string
  permission_key: string
  org_id?: string | null
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const req = HttpClientRequest.del('/api/dashboard/role-permissions').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(req)
    const resBody = yield* response.json
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to manage permissions.'),
      )
    }
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErr(resBody)))
    }
  })

export type AuditLogEntry = {
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
}

/** Fetch audit log with query params. Requires HttpClient. */
export const fetchAuditLogEffect = (params: {
  limit: number
  offset: number
  from?: string
  to?: string
  outcome?: string
  event_kind?: string
  action?: string
  reason?: string
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
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
    const data = body as { entries?: AuditLogEntry[]; total?: number }
    return {
      entries: (data.entries ?? []) as AuditLogEntry[],
      total: data.total ?? 0,
    }
  })

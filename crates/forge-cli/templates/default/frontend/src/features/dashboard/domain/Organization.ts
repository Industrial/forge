import { Data } from 'effect'

/**
 * Organization entity. Data class for structural equality and tagging.
 * Used by dashboard pages (e.g. OrganizationsPage via EntityApi) and Dashboard service.
 */
export class Organization extends Data.TaggedClass('Organization')<{
  readonly id: string
  readonly name: string
  readonly slug: string
  readonly created_at?: string
  readonly updated_at?: string
}> {}

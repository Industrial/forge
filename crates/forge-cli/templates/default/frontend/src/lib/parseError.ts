/**
 * Parse an error message from an API response body.
 * Handles common shapes: { error: string }, { message: string }.
 */
export function parseError(body: unknown): string {
  if (typeof body !== 'object' || body === null) {
    return 'Request failed.'
  }
  if (
    'message' in body &&
    typeof (body as { message: unknown }).message === 'string'
  ) {
    return (body as { message: string }).message
  }
  if (
    'error' in body &&
    typeof (body as { error: unknown }).error === 'string'
  ) {
    return (body as { error: string }).error
  }
  if ('error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

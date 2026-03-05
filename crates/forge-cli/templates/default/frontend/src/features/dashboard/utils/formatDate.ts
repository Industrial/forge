export function formatDate(iso: string | undefined): string {
  if (iso == null || iso === '') return '—'
  try {
    return new Date(iso).toLocaleString()
  } catch {
    return iso
  }
}

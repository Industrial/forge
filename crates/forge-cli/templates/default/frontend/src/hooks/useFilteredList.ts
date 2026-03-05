import { useMemo, useState, type Dispatch, type SetStateAction } from 'react'

/**
 * Holds filter state and derives a filtered list. Use for list pages with
 * multiple filter fields (e.g. name, slug) without wiring each field manually.
 *
 * @param items - Full list to filter.
 * @param initialFilters - Initial filter object (e.g. `{ filterName: '', filterSlug: '' }`).
 * @param predicate - (item, filters) => true to keep the item.
 * @returns [filters, setFilters, filteredItems]
 */
export function useFilteredList<T, F extends Record<string, unknown>>(
  items: readonly T[],
  initialFilters: F,
  predicate: (item: T, filters: F) => boolean,
): [F, Dispatch<SetStateAction<F>>, T[]] {
  const [filters, setFilters] = useState<F>(initialFilters)
  const filteredItems = useMemo(
    () => items.filter((item) => predicate(item, filters)),
    [items, filters, predicate],
  )
  return [filters, setFilters, filteredItems]
}

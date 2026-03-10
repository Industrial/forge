/**
 * Hook for server-driven list with pagination and filters.
 * Runs a fetch effect when page, rowsPerPage, filters, or deps change;
 * exposes items, total, loading, error, refresh, and pagination/filter state.
 *
 * @see GenericEntityCrud (list state + runStreamInto pattern)
 * @see AuditLogPage (server pagination + filters)
 */

import { useEffect, useState, useCallback } from 'react'
import {
  useEffectState,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  isSuccess,
  isFailure,
  isPending,
} from 'react-effect-hooks'
import { Effect } from 'effect'

import { getApplicationLayer, type AppServices } from '@/lib/appLayer'
import { useTablePaginationDefaults } from '@/hooks/useTablePaginationDefaults'

export type ServerListParams<F> = {
  offset: number
  limit: number
  filters: F
}

export type ServerListResult<T> = {
  items: readonly T[]
  total: number
}

export type UseServerListOptions<T, F> = {
  /** Effect that fetches one page of items. Receives offset, limit, and current filters. */
  fetch: (
    params: ServerListParams<F>,
  ) => Effect.Effect<ServerListResult<T>, Error, AppServices>
  /** Initial filter values. */
  initialFilters: F
  /** Optional dependency list; refetch when these change (e.g. liveRefreshTrigger). */
  deps?: React.DependencyList
}

export type UseServerListReturn<T, F> = {
  items: readonly T[]
  total: number
  loading: boolean
  error: Error | null
  errorMessage: string | null
  refresh: () => void
  page: number
  setPage: (page: number | ((prev: number) => number)) => void
  rowsPerPage: number
  setRowsPerPage: (value: number) => void
  rowsPerPageOptions: number[]
  filters: F
  setFilters: (value: F | ((prev: F) => F)) => void
}

type ListState<T> = AsyncState<ServerListResult<T>, Error>

export function useServerList<T, F>({
  fetch: fetchEffectFn,
  initialFilters,
  deps: _deps = [],
}: UseServerListOptions<T, F>): UseServerListReturn<T, F> {
  const { defaultRowsPerPage, rowsPerPageOptions } =
    useTablePaginationDefaults()

  const [listState, , setListStateAsEffect] = useEffectState<
    ListState<T>,
    never,
    never
  >(idle<ServerListResult<T>, Error>())

  const [page, setPageState] = useState(0)
  const [rowsPerPage, setRowsPerPageState] = useState(defaultRowsPerPage)
  const [filters, setFiltersState] = useState<F>(initialFilters)

  const items = isSuccess(listState) ? listState.value.items : []
  const total = isSuccess(listState) ? listState.value.total : 0
  const loading = isPending(listState)
  const error: Error | null = isFailure(listState) ? listState.error : null
  const errorMessage =
    error instanceof Error
      ? error.message
      : error != null
        ? String(error)
        : null

  const refreshEffect = Effect.gen(function* () {
    const listEffect = fetchEffectFn({
      offset: page * rowsPerPage,
      limit: rowsPerPage,
      filters,
    })
    yield* runStreamInto(
      streamWithPendingState(listEffect),
      setListStateAsEffect,
    )
  })

  const refresh = useCallback(() => {
    Effect.runPromise(
      refreshEffect.pipe(Effect.provide(getApplicationLayer())),
    ).catch(() => {})
  }, [refreshEffect.pipe])

  useEffect(() => {
    Effect.runPromise(
      refreshEffect.pipe(Effect.provide(getApplicationLayer())),
    ).catch(() => {})
  }, [refreshEffect.pipe])

  useEffect(() => {
    setRowsPerPageState(defaultRowsPerPage)
    setPageState(0)
  }, [defaultRowsPerPage])

  const setPage = useCallback((value: number | ((prev: number) => number)) => {
    setPageState((prev) => (typeof value === 'function' ? value(prev) : value))
  }, [])

  const setRowsPerPage = useCallback((value: number) => {
    setRowsPerPageState(value)
    setPageState(0)
  }, [])

  const setFilters = useCallback((value: F | ((prev: F) => F)) => {
    setFiltersState(value)
    setPageState(0)
  }, [])

  return {
    items,
    total,
    loading,
    error,
    errorMessage,
    refresh,
    page,
    setPage,
    rowsPerPage,
    setRowsPerPage,
    rowsPerPageOptions,
    filters,
    setFilters,
  }
}

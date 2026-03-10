import type { ReactNode } from 'react'
import Box from '@mui/material/Box'
import TablePagination from '@mui/material/TablePagination'

import type { DataTablePagination } from '@/components/DataTable'

export type DataListMobileProps<T> = {
  /** Items to render (e.g. current page slice or full filtered list). */
  items: readonly T[]
  /** Row key for React. */
  getKey: (item: T) => string
  /** Renders each item as a card (e.g. OrganizationCard, UserCard). */
  renderItem: (item: T) => ReactNode
  /** Optional pagination; when provided, shows pagination controls below the list. */
  pagination?: DataTablePagination
  /** Accessible label for the list. */
  ariaLabel?: string
  /** Optional test id for the list container. */
  dataTestId?: string
}

/**
 * Mobile-friendly list of cards. Use with the same data and optional pagination
 * as DataTable so dashboard pages can switch between DataTable (desktop) and
 * DataListMobile (mobile) with useIsMobile().
 */
export default function DataListMobile<T>({
  items,
  getKey,
  renderItem,
  pagination,
  ariaLabel = 'List',
  dataTestId = 'data-list-mobile',
}: DataListMobileProps<T>) {
  return (
    <>
      <Box
        component="ul"
        aria-label={ariaLabel}
        data-testid={dataTestId}
        sx={{
          listStyle: 'none',
          m: 0,
          p: 0,
          display: 'flex',
          flexDirection: 'column',
          gap: 1.5,
        }}
      >
        {items.map((item) => (
          <Box key={getKey(item)} component="li">
            {renderItem(item)}
          </Box>
        ))}
      </Box>
      {pagination != null && (
        <Box component="div" sx={{ display: 'block' }}>
          <TablePagination
            component="div"
            count={pagination.totalCount}
            page={pagination.page}
            onPageChange={pagination.onPageChange}
            rowsPerPage={pagination.rowsPerPage}
            onRowsPerPageChange={pagination.onRowsPerPageChange}
            rowsPerPageOptions={pagination.rowsPerPageOptions}
            labelRowsPerPage="Rows per page:"
          />
        </Box>
      )}
    </>
  )
}

import EntityTableRow from '@/components/EntityTableRow'
import type { Organization } from '@/features/dashboard/domain/Organization'
import { formatDate } from '@/features/dashboard/utils/formatDate'
import { useComponentLogger } from '@/hooks'

export type OrganizationTableRowProps = {
  org: Organization
  canWrite: boolean
  onEdit: (org: Organization) => void
  onDelete: (id: string) => void
  isDeleting: boolean
}

export default function OrganizationTableRow({
  org,
  canWrite,
  onEdit,
  onDelete,
  isDeleting,
}: OrganizationTableRowProps) {
  useComponentLogger('OrganizationTableRow')
  return (
    <EntityTableRow<Organization>
      item={org}
      getRowId={(o) => o.id}
      canEditDelete={canWrite}
      onEdit={onEdit}
      onDelete={(o) => onDelete(o.id)}
      isDeleting={isDeleting}
      columns={[
        { key: 'name', render: (o) => o.name, cellSx: { fontWeight: 500 } },
        { key: 'slug', render: (o) => o.slug },
        {
          key: 'created_at',
          render: (o) => formatDate(o.created_at),
          cellSx: { whiteSpace: 'nowrap' },
        },
        {
          key: 'updated_at',
          render: (o) => formatDate(o.updated_at),
          cellSx: { whiteSpace: 'nowrap' },
        },
      ]}
    />
  )
}

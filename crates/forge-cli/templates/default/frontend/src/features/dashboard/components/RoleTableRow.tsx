import EntityTableRow from '@/components/EntityTableRow'
import type { Role } from '@/features/dashboard/domain/Role'

export type RoleTableRowProps = {
  role: Role
  orgName: string
  onEdit: (role: Role) => void
  onDelete: (id: string) => void
  onView?: (role: Role) => void
  canWrite: boolean
  isDeleting: boolean
}

export default function RoleTableRow({
  role,
  orgName,
  onEdit,
  onDelete,
  onView,
  canWrite,
  isDeleting,
}: RoleTableRowProps) {
  const onRowClick = () => {
    if (onView) onView(role)
    else if (canWrite) onEdit(role)
  }
  return (
    <EntityTableRow<Role>
      item={role}
      getRowId={(r) => r.name}
      testIdPrefix="role-row"
      onRowClick={onRowClick}
      canEditDelete={canWrite}
      onEdit={onEdit}
      onDelete={(r) => onDelete(r.id)}
      isDeleting={isDeleting}
      columns={[
        { key: 'orgName', render: () => orgName },
        {
          key: 'name',
          render: (r) => r.display_name ?? r.name ?? '—',
          cellSx: { fontWeight: 500 },
        },
      ]}
    />
  )
}

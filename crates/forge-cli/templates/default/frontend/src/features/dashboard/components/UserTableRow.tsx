import EntityTableRow from '../../../components/EntityTableRow'
import { formatDate } from '../../../features/dashboard/utils/formatDate'
import { membershipsSummary } from '../../../features/dashboard/utils/membershipsSummary'

export type UserRow = {
  id: string
  email: string
  is_active: boolean
  is_admin: boolean
  created_at: string
  memberships: readonly { org_name: string; roles?: readonly string[] | null }[]
}

export type UserTableRowProps = {
  user: UserRow
  canWrite: boolean
  onEdit: (user: UserRow) => void
  onDelete: (id: string) => void
  onView?: (user: UserRow) => void
  isDeleting: boolean
}

export default function UserTableRow({
  user,
  canWrite,
  onEdit,
  onDelete,
  onView,
  isDeleting,
}: UserTableRowProps) {
  const onRowClick = () => {
    if (onView) {
      onView(user)
    } else if (canWrite) {
      onEdit(user)
    }
  }
  return (
    <EntityTableRow<UserRow>
      item={user}
      getRowId={(u) => u.id}
      testIdPrefix="user-row"
      onRowClick={onRowClick}
      canEditDelete={canWrite}
      onEdit={onEdit}
      onDelete={(u) => onDelete(u.id)}
      isDeleting={isDeleting}
      columns={[
        {
          key: 'email',
          render: (u) => u.email,
          cellSx: { fontWeight: 500 },
        },
        {
          key: 'memberships',
          render: (u) => membershipsSummary(u.memberships),
          cellSx: { maxWidth: 280 },
        },
        {
          key: 'is_active',
          render: (u) => (u.is_active ? 'Yes' : 'No'),
        },
        {
          key: 'is_admin',
          render: (u) => (u.is_admin ? 'Yes' : 'No'),
        },
        {
          key: 'created_at',
          render: (u) => formatDate(u.created_at),
          cellSx: { whiteSpace: 'nowrap' },
        },
      ]}
    />
  )
}

import EntityTableRow from '@/components/EntityTableRow'
import type { Assignment } from '../domain/Assignment'

export type AssignmentTableRowProps = {
  assignment: Assignment
  onDelete: (assignment: Assignment) => void
  isDeleting: boolean
}

export default function AssignmentTableRow({
  assignment,
  onDelete,
  isDeleting,
}: AssignmentTableRowProps) {
  const getRowId = (a: Assignment) =>
    `${a.scope}-${a.role_name}-${a.permission_key}`
  return (
    <EntityTableRow<Assignment>
      item={assignment}
      getRowId={getRowId}
      canDeleteOnly
      onDelete={onDelete}
      isDeleting={isDeleting}
      deleteAriaLabel="Remove"
      columns={[
        { key: 'scope', render: (a) => a.scope },
        { key: 'role_name', render: (a) => a.role_name },
        { key: 'permission_key', render: (a) => a.permission_key },
      ]}
    />
  )
}

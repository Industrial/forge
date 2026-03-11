import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Paper from '@mui/material/Paper'
import VisibilityIcon from '@mui/icons-material/Visibility'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'

import type { Role } from '../../../features/dashboard/domain/Role'
import type { Organization } from '../../../features/dashboard/domain/Organization'

export type RoleCardProps = {
  role: Role
  organizations: readonly Organization[]
  canWrite: boolean
  onView: (role: Role) => void
  onEdit: (role: Role) => void
  onDelete: (id: string) => void
  isDeleting: boolean
}

function getOrgName(
  role: Role,
  organizations: readonly Organization[],
): string {
  return (
    role.org_name ??
    organizations.find((o) => o.id === role.org_id)?.name ??
    role.org_id
  )
}

export default function RoleCard({
  role,
  organizations,
  canWrite,
  onView,
  onEdit,
  onDelete,
  isDeleting,
}: RoleCardProps) {
  const displayName = role.display_name ?? role.name ?? '—'
  return (
    <Paper sx={{ p: 2 }}>
      <Typography variant="subtitle1" fontWeight={600}>
        {displayName}
      </Typography>
      <Typography variant="body2" color="text.secondary">
        {getOrgName(role, organizations)}
      </Typography>
      <Box
        sx={{
          mt: 2,
          display: 'flex',
          gap: 0.5,
          justifyContent: 'flex-end',
        }}
      >
        <Button
          size="medium"
          startIcon={<VisibilityIcon />}
          onClick={() => onView(role)}
        >
          View
        </Button>
        {canWrite && (
          <>
            <Button
              size="medium"
              startIcon={<EditIcon />}
              onClick={() => onEdit(role)}
            >
              Edit
            </Button>
            <Button
              size="medium"
              color="error"
              startIcon={<DeleteIcon />}
              onClick={() => onDelete(role.id)}
              disabled={isDeleting}
            >
              Delete
            </Button>
          </>
        )}
      </Box>
    </Paper>
  )
}

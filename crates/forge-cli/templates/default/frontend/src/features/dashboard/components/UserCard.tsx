import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Paper from '@mui/material/Paper'
import VisibilityIcon from '@mui/icons-material/Visibility'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'

import type { User } from '@/features/dashboard/domain/User'
import { formatDate } from '@/features/dashboard/utils/formatDate'
import { membershipsSummary } from '@/features/dashboard/utils/membershipsSummary'

export type UserCardProps = {
  user: User
  canWrite: boolean
  onView: (user: User) => void
  onEdit: (user: User) => void
  onDelete: (id: string) => void
  isDeleting: boolean
}

export default function UserCard({
  user,
  canWrite,
  onView,
  onEdit,
  onDelete,
  isDeleting,
}: UserCardProps) {
  return (
    <Paper sx={{ p: 2 }}>
      <Typography variant="subtitle1" fontWeight={600}>
        {user.email}
      </Typography>
      <Typography variant="body2" color="text.secondary">
        {membershipsSummary(user.memberships)}
      </Typography>
      <Typography
        variant="caption"
        color="text.secondary"
        display="block"
        sx={{ mt: 0.5 }}
      >
        Active: {user.is_active ? 'Yes' : 'No'} · Admin:{' '}
        {user.is_admin ? 'Yes' : 'No'} · Created {formatDate(user.created_at)}
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
          onClick={() => onView(user)}
        >
          View
        </Button>
        {canWrite && (
          <>
            <Button
              size="medium"
              startIcon={<EditIcon />}
              onClick={() => onEdit(user)}
            >
              Edit
            </Button>
            <Button
              size="medium"
              color="error"
              startIcon={<DeleteIcon />}
              onClick={() => onDelete(user.id)}
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

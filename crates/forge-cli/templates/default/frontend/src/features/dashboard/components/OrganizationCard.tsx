import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Paper from '@mui/material/Paper'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import type { Organization } from '../domain/Organization'
import { formatDate } from '../utils/formatDate'

export type OrganizationCardProps = {
  org: Organization
  canWrite: boolean
  onEdit: (org: Organization) => void
  onDelete: (id: string) => void
  isDeleting: boolean
}

export default function OrganizationCard({
  org,
  canWrite,
  onEdit,
  onDelete,
  isDeleting,
}: OrganizationCardProps) {
  return (
    <Paper sx={{ p: 2 }}>
      <Typography variant="subtitle1" fontWeight={600}>
        {org.name}
      </Typography>
      <Typography variant="body2" color="text.secondary">
        {org.slug}
      </Typography>
      <Typography
        variant="caption"
        color="text.secondary"
        display="block"
        sx={{ mt: 0.5 }}
      >
        Created {formatDate(org.created_at)} · Updated{' '}
        {formatDate(org.updated_at)}
      </Typography>
      {canWrite && (
        <Box
          sx={{
            mt: 2,
            display: 'flex',
            gap: 0.5,
            justifyContent: 'flex-end',
          }}
        >
          <Button
            size="small"
            startIcon={<EditIcon />}
            onClick={() => onEdit(org)}
          >
            Edit
          </Button>
          <Button
            size="small"
            color="error"
            startIcon={<DeleteIcon />}
            onClick={() => onDelete(org.id)}
            disabled={isDeleting}
          >
            Delete
          </Button>
        </Box>
      )}
    </Paper>
  )
}

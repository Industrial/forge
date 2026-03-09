import Paper from '@mui/material/Paper'
import Typography from '@mui/material/Typography'
import { useComponentLogger } from '@/hooks'

export type EmptyStateProps = {
  message: string
}

export default function EmptyState({ message }: EmptyStateProps) {
  useComponentLogger('EmptyState')
  return (
    <Paper sx={{ p: 3, textAlign: 'center' }}>
      <Typography color="text.secondary">{message}</Typography>
    </Paper>
  )
}

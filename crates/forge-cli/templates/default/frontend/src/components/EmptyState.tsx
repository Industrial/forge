import Paper from '@mui/material/Paper'
import Typography from '@mui/material/Typography'

export type EmptyStateProps = {
  message: string
}

export default function EmptyState({ message }: EmptyStateProps) {
  return (
    <Paper sx={{ p: 3, textAlign: 'center' }}>
      <Typography color="text.secondary">{message}</Typography>
    </Paper>
  )
}

import Alert from '@mui/material/Alert'
import { useComponentLogger } from '@/hooks'

export type ErrorAlertProps = {
  message: string
  onClose: () => void
}

export default function ErrorAlert({ message, onClose }: ErrorAlertProps) {
  useComponentLogger('ErrorAlert')
  return (
    <Alert severity="error" sx={{ mb: 2 }} onClose={onClose}>
      {message}
    </Alert>
  )
}

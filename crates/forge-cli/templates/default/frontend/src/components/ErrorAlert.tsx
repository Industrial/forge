import Alert from '@mui/material/Alert'

export type ErrorAlertProps = {
  message: string
  onClose: () => void
}

export default function ErrorAlert({ message, onClose }: ErrorAlertProps) {
  return (
    <Alert severity="error" sx={{ mb: 2 }} onClose={onClose}>
      {message}
    </Alert>
  )
}

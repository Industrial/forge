import type { ReactNode } from 'react'
import Button from '@mui/material/Button'
import { useComponentLogger } from '@/hooks'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import type { SxProps, Theme } from '@mui/material/styles'

export type FormDialogProps = {
  open: boolean
  onClose: () => void
  title: string
  submitLabel: string
  submittingLabel?: string
  onSubmit: () => void
  submitDisabled: boolean
  submitting?: boolean
  children: ReactNode
  maxWidth?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  fullWidth?: boolean
  contentSx?: SxProps<Theme>
  'data-testid'?: string
  submitButtonTestId?: string
}

export default function FormDialog({
  open,
  onClose,
  title,
  submitLabel,
  submittingLabel,
  onSubmit,
  submitDisabled,
  submitting = false,
  children,
  maxWidth = 'sm',
  fullWidth = true,
  contentSx,
  'data-testid': testId,
  submitButtonTestId,
}: FormDialogProps) {
  useComponentLogger('FormDialog')
  const handleClose = () => {
    if (!submitting) onClose()
  }
  const buttonLabel =
    submitting && submittingLabel ? submittingLabel : submitLabel

  return (
    <Dialog
      open={open}
      onClose={handleClose}
      maxWidth={maxWidth}
      fullWidth={fullWidth}
      data-testid={testId}
    >
      <DialogTitle>{title}</DialogTitle>
      <DialogContent sx={contentSx}>{children}</DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={submitting}>
          Cancel
        </Button>
        <Button
          variant="contained"
          onClick={onSubmit}
          disabled={submitDisabled || submitting}
          data-testid={submitButtonTestId}
        >
          {buttonLabel}
        </Button>
      </DialogActions>
    </Dialog>
  )
}

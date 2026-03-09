import TextField from '@mui/material/TextField'
import type { TextFieldProps } from '@mui/material/TextField'

export type FormTextFieldProps = Omit<
  TextFieldProps,
  'value' | 'onChange' | 'onBlur' | 'error' | 'helperText'
> & {
  value: string
  onChange: (value: string) => void
  onBlur?: () => void
  error?: boolean
  helperText?: string
}

export default function FormTextField({
  value,
  onChange,
  onBlur,
  error = false,
  helperText,
  ...rest
}: FormTextFieldProps) {
  return (
    <TextField
      {...rest}
      fullWidth
      value={value}
      onChange={(e) => onChange(e.target.value)}
      onBlur={onBlur}
      error={error}
      helperText={helperText}
    />
  )
}

import type { ReactNode } from 'react'
import Box from '@mui/material/Box'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Paper from '@mui/material/Paper'
import Select from '@mui/material/Select'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'

export type FilterFieldText = {
  type: 'text'
  key: string
  label: string
  placeholder?: string
  minWidth?: number
  inputType?: 'text' | 'search'
  /** For e2e: data-testid on the input (e.g. "users-filter-input"). */
  dataTestId?: string
}

export type FilterFieldSelect = {
  type: 'select'
  key: string
  label: string
  options: ReadonlyArray<{ value: string; label: string }>
  minWidth?: number
}

export type FilterFieldDate = {
  type: 'date'
  key: string
  label: string
  minWidth?: number
}

export type FilterField = FilterFieldText | FilterFieldSelect | FilterFieldDate

export type FiltersPanelProps =
  | {
      /** Panel title (e.g. "Filters"). Default "Filters". */
      title?: string
      children: ReactNode
    }
  | {
      title?: string
      fields: readonly FilterField[]
      values: Record<string, string>
      onChange: (key: string, value: string) => void
      /** Optional content after the fields (e.g. Apply/Reset buttons). */
      extra?: ReactNode
    }

/**
 * Shared panel for filter controls: Paper with optional title and a flex wrap box.
 * Use for Organizations, AuditLog, Users filter sections.
 * Supports either children or a declarative field config (fields, values, onChange, extra).
 */
export default function FiltersPanel(props: FiltersPanelProps) {
  const title = props.title ?? 'Filters'
  const hasChildren = 'children' in props

  return (
    <Paper
      sx={{
        width: '100%',
        maxWidth: '100%',
        boxSizing: 'border-box',
        p: 2,
        mb: 2,
      }}
    >
      <Typography variant="subtitle2" gutterBottom>
        {title}
      </Typography>
      <Box
        sx={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: 2,
          alignItems: 'flex-end',
        }}
      >
        {!hasChildren ? (
          <>
            {props.fields.map((field) => {
              if (field.type === 'text') {
                return (
                  <TextField
                    key={field.key}
                    label={field.label}
                    type={field.inputType ?? 'text'}
                    size="small"
                    value={props.values[field.key] ?? ''}
                    onChange={(e) => props.onChange(field.key, e.target.value)}
                    placeholder={field.placeholder}
                    sx={{ minWidth: field.minWidth ?? 160 }}
                    inputProps={
                      field.dataTestId
                        ? { 'data-testid': field.dataTestId }
                        : undefined
                    }
                  />
                )
              }
              if (field.type === 'select') {
                return (
                  <FormControl
                    key={field.key}
                    size="small"
                    sx={{ minWidth: field.minWidth ?? 120 }}
                  >
                    <InputLabel>{field.label}</InputLabel>
                    <Select
                      label={field.label}
                      value={props.values[field.key] ?? ''}
                      onChange={(e) =>
                        props.onChange(field.key, e.target.value)
                      }
                    >
                      {field.options.map((opt) => (
                        <MenuItem key={opt.value} value={opt.value}>
                          {opt.label}
                        </MenuItem>
                      ))}
                    </Select>
                  </FormControl>
                )
              }
              if (field.type === 'date') {
                return (
                  <TextField
                    key={field.key}
                    label={field.label}
                    type="date"
                    size="small"
                    value={props.values[field.key] ?? ''}
                    onChange={(e) => props.onChange(field.key, e.target.value)}
                    InputLabelProps={{ shrink: true }}
                    sx={{ minWidth: field.minWidth ?? 140 }}
                  />
                )
              }
              return null
            })}
            {props.extra}
          </>
        ) : (
          props.children
        )}
      </Box>
    </Paper>
  )
}

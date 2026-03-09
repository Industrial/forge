import type { ReactNode } from 'react'
import TextField from '@mui/material/TextField'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Select from '@mui/material/Select'
import { useComponentLogger } from '@/hooks'
import FiltersPanel from './FiltersPanel'

export type FilterFieldText = {
  type: 'text'
  key: string
  label: string
  placeholder?: string
  minWidth?: number
  inputType?: 'text' | 'search'
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

export type FilterPanelFromConfigProps = {
  title?: string
  fields: readonly FilterField[]
  values: Record<string, string>
  onChange: (key: string, value: string) => void
  /** Optional content after the fields (e.g. Apply/Reset buttons). */
  extra?: ReactNode
}

/**
 * Renders filter controls from a declarative field config.
 */
export default function FilterPanelFromConfig({
  title = 'Filters',
  fields,
  values,
  onChange,
  extra,
}: FilterPanelFromConfigProps) {
  useComponentLogger('FilterPanelFromConfig')
  return (
    <FiltersPanel title={title}>
      {fields.map((field) => {
        if (field.type === 'text') {
          return (
            <TextField
              key={field.key}
              label={field.label}
              type={field.inputType ?? 'text'}
              size="small"
              value={values[field.key] ?? ''}
              onChange={(e) => onChange(field.key, e.target.value)}
              placeholder={field.placeholder}
              sx={{ minWidth: field.minWidth ?? 160 }}
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
                value={values[field.key] ?? ''}
                onChange={(e) => onChange(field.key, e.target.value)}
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
              value={values[field.key] ?? ''}
              onChange={(e) => onChange(field.key, e.target.value)}
              InputLabelProps={{ shrink: true }}
              sx={{ minWidth: field.minWidth ?? 140 }}
            />
          )
        }
        return null
      })}
      {extra}
    </FiltersPanel>
  )
}

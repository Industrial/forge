import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Select from '@mui/material/Select'
import FiltersPanel from '../../../components/FiltersPanel'

const OUTCOMES = ['success', 'failure', 'allowed', 'denied'] as const
const EVENT_KINDS = ['auth', 'authz', 'mutation', 'custom'] as const
const ACTIONS = ['read', 'create', 'update', 'delete', 'manage'] as const

export { OUTCOMES, EVENT_KINDS, ACTIONS }

export type AuditLogFiltersProps = {
  from: string
  to: string
  outcome: string
  eventKind: string
  action: string
  reason: string
  onFromChange: (value: string) => void
  onToChange: (value: string) => void
  onOutcomeChange: (value: string) => void
  onEventKindChange: (value: string) => void
  onActionChange: (value: string) => void
  onReasonChange: (value: string) => void
  onApply: () => void
  onReset: () => void
}

export default function AuditLogFilters({
  from,
  to,
  outcome,
  eventKind,
  action,
  reason,
  onFromChange,
  onToChange,
  onOutcomeChange,
  onEventKindChange,
  onActionChange,
  onReasonChange,
  onApply,
  onReset,
}: AuditLogFiltersProps) {
  return (
    <FiltersPanel>
      <TextField
        label="From (date)"
        type="date"
        size="small"
        value={from}
        onChange={(e) => onFromChange(e.target.value)}
        InputLabelProps={{ shrink: true }}
        sx={{ minWidth: 140 }}
      />
      <TextField
        label="To (date)"
        type="date"
        size="small"
        value={to}
        onChange={(e) => onToChange(e.target.value)}
        InputLabelProps={{ shrink: true }}
        sx={{ minWidth: 140 }}
      />
      <FormControl size="small" sx={{ minWidth: 120 }}>
        <InputLabel>Outcome</InputLabel>
        <Select
          value={outcome}
          label="Outcome"
          onChange={(e) => onOutcomeChange(e.target.value)}
        >
          <MenuItem value="">All</MenuItem>
          {OUTCOMES.map((o) => (
            <MenuItem key={o} value={o}>
              {o}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl size="small" sx={{ minWidth: 120 }}>
        <InputLabel>Event kind</InputLabel>
        <Select
          value={eventKind}
          label="Event kind"
          onChange={(e) => onEventKindChange(e.target.value)}
        >
          <MenuItem value="">All</MenuItem>
          {EVENT_KINDS.map((k) => (
            <MenuItem key={k} value={k}>
              {k}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl size="small" sx={{ minWidth: 100 }}>
        <InputLabel>Action</InputLabel>
        <Select
          value={action}
          label="Action"
          onChange={(e) => onActionChange(e.target.value)}
        >
          <MenuItem value="">All</MenuItem>
          {ACTIONS.map((a) => (
            <MenuItem key={a} value={a}>
              {a}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <TextField
        label="Reason contains"
        size="small"
        value={reason}
        onChange={(e) => onReasonChange(e.target.value)}
        placeholder="Search in reason"
        sx={{ minWidth: 160 }}
      />
      <Button variant="contained" onClick={onApply}>
        Apply
      </Button>
      <Button onClick={onReset}>Reset</Button>
    </FiltersPanel>
  )
}

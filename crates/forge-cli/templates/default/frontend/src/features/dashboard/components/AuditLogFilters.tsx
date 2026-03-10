import FiltersPanel from '@/components/FiltersPanel'
import type { FilterField } from '@/components/FiltersPanel'

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
}

const FIELDS: readonly FilterField[] = [
  { type: 'date', key: 'from', label: 'From (date)', minWidth: 140 },
  { type: 'date', key: 'to', label: 'To (date)', minWidth: 140 },
  {
    type: 'select',
    key: 'outcome',
    label: 'Outcome',
    minWidth: 120,
    options: [
      { value: '', label: 'All' },
      ...OUTCOMES.map((o) => ({ value: o, label: o })),
    ],
  },
  {
    type: 'select',
    key: 'eventKind',
    label: 'Event kind',
    minWidth: 120,
    options: [
      { value: '', label: 'All' },
      ...EVENT_KINDS.map((k) => ({ value: k, label: k })),
    ],
  },
  {
    type: 'select',
    key: 'action',
    label: 'Action',
    minWidth: 100,
    options: [
      { value: '', label: 'All' },
      ...ACTIONS.map((a) => ({ value: a, label: a })),
    ],
  },
  {
    type: 'text',
    key: 'reason',
    label: 'Reason contains',
    placeholder: 'Search in reason',
    minWidth: 160,
    dataTestId: 'audit-log-filter-input',
  },
]

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
}: AuditLogFiltersProps) {
  const values = { from, to, outcome, eventKind, action, reason }
  const onChange = (key: string, value: string) => {
    if (key === 'from') {
      onFromChange(value)
    } else if (key === 'to') {
      onToChange(value)
    } else if (key === 'outcome') {
      onOutcomeChange(value)
    } else if (key === 'eventKind') {
      onEventKindChange(value)
    } else if (key === 'action') {
      onActionChange(value)
    } else if (key === 'reason') {
      onReasonChange(value)
    }
  }
  return <FiltersPanel fields={FIELDS} values={values} onChange={onChange} />
}

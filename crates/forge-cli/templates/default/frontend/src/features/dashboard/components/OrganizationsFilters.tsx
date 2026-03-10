import FiltersPanel from '@/components/FiltersPanel'
import type { FilterField } from '@/components/FiltersPanel'

export type OrgOption = { id: string; name: string }

export type OrganizationsFiltersProps = {
  filterName: string
  filterSlug: string
  onFilterNameChange: (value: string) => void
  onFilterSlugChange: (value: string) => void
}

const FIELDS: readonly FilterField[] = [
  {
    type: 'text',
    key: 'name',
    label: 'Name',
    placeholder: 'Search by name',
    minWidth: 200,
  },
  {
    type: 'text',
    key: 'slug',
    label: 'Slug',
    placeholder: 'Search by slug',
    minWidth: 160,
  },
] as const

export default function OrganizationsFilters({
  filterName,
  filterSlug,
  onFilterNameChange,
  onFilterSlugChange,
}: OrganizationsFiltersProps) {
  const values = { name: filterName, slug: filterSlug }
  const onChange = (key: string, value: string) => {
    if (key === 'name') onFilterNameChange(value)
    else if (key === 'slug') onFilterSlugChange(value)
  }
  return <FiltersPanel fields={FIELDS} values={values} onChange={onChange} />
}

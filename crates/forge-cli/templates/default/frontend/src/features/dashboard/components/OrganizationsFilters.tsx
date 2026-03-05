import TextField from '@mui/material/TextField'
import FiltersPanel from '../../../components/FiltersPanel'

export type OrganizationsFiltersProps = {
  filterName: string
  filterSlug: string
  onFilterNameChange: (value: string) => void
  onFilterSlugChange: (value: string) => void
}

export default function OrganizationsFilters({
  filterName,
  filterSlug,
  onFilterNameChange,
  onFilterSlugChange,
}: OrganizationsFiltersProps) {
  return (
    <FiltersPanel>
      <TextField
          label="Name"
          size="small"
          value={filterName}
          onChange={(e) => onFilterNameChange(e.target.value)}
          placeholder="Search by name"
          sx={{ minWidth: 200 }}
        />
        <TextField
          label="Slug"
          size="small"
          value={filterSlug}
          onChange={(e) => onFilterSlugChange(e.target.value)}
          placeholder="Search by slug"
          sx={{ minWidth: 160 }}
        />
    </FiltersPanel>
  )
}

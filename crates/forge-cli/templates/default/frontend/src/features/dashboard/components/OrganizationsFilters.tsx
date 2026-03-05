import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import TextField from '@mui/material/TextField'
import Paper from '@mui/material/Paper'

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
    <Paper sx={{ p: 2, mb: 2 }}>
      <Typography variant="subtitle2" gutterBottom>
        Filters
      </Typography>
      <Box
        sx={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: 2,
          alignItems: 'flex-end',
        }}
      >
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
      </Box>
    </Paper>
  )
}

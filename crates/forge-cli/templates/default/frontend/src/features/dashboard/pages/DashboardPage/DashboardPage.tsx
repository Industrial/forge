import Box from '@mui/material/Box'
import PageHeader from '../../../../components/PageHeader'

export default function DashboardPage() {
  return (
    <Box data-testid="dashboard-page">
      <PageHeader
        title="Dashboard"
        description="Welcome to your dashboard."
        data-testid="dashboard-heading"
      />
    </Box>
  )
}

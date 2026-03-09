import Box from '@mui/material/Box'
import PageHeader from '@/components/PageHeader'
import { useComponentLogger } from '@/hooks'

export default function DashboardPage() {
  useComponentLogger('DashboardPage')
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

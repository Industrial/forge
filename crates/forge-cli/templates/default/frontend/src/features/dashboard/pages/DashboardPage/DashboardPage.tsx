import Typography from '@mui/material/Typography'

export default function DashboardPage() {
  return (
    <>
      <Typography
        variant="h4"
        component="h1"
        gutterBottom
        data-testid="dashboard-heading"
      >
        Dashboard
      </Typography>
      <Typography>Welcome to your dashboard.</Typography>
    </>
  )
}

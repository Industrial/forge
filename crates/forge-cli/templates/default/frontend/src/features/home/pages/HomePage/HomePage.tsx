import Typography from '@mui/material/Typography'
import PageHeader from '../../../../components/PageHeader'
import { useComponentLogger } from '@/hooks'

export default function HomePage() {
  useComponentLogger('HomePage')
  return (
    <>
      <PageHeader title="Home" />
      <Typography color="text.secondary" data-testid="home-welcome">
        Welcome. You are logged in.
      </Typography>
    </>
  )
}

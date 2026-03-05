import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Typography from '@mui/material/Typography'
import PageHeader from '../../../../components/PageHeader'
import { useAuthentication } from '../../../../context/AuthenticationContext'

export default function ProfilePage() {
  const { user, logout, loading } = useAuthentication()

  return (
    <>
      <PageHeader title="Profile" />
      {user != null && (
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
          <Typography color="text.secondary">
            Email: {user.email}
          </Typography>
          <Box>
            <Button
              type="button"
              variant="outlined"
              onClick={() => logout()}
              disabled={loading}
            >
              Log out
            </Button>
          </Box>
        </Box>
      )}
    </>
  )
}

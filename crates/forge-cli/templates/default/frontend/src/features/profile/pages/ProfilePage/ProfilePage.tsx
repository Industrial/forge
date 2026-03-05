import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Typography from '@mui/material/Typography'
import { useSession } from '../../../../context/Session'

export default function ProfilePage() {
  const { user } = useSession()

  return (
    <>
      <Typography variant="h4" component="h1" gutterBottom>
        Profile
      </Typography>
      {user != null && (
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
          <Typography>Email: {user.email}</Typography>
          <form action="/api/auth/logout" method="get" target="_top">
            <Button type="submit" variant="outlined">
              Log out
            </Button>
          </form>
        </Box>
      )}
    </>
  )
}

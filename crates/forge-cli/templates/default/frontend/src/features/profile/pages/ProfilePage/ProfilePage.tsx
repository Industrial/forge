import { useState } from 'react'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Typography from '@mui/material/Typography'
import { Effect, Option } from 'effect'
import PageHeader from '@/components/PageHeader'
import { getApplicationLayer } from '@/lib/appLayer'
import { useAuthenticationStateReactiveStore } from '@/features/authentication/stores'
import { Authentication } from '@/features/authentication/services/Authentication'

export default function ScopePage() {
  const authentication = useAuthenticationStateReactiveStore()
  const user = Option.getOrElse(authentication.user, () => null)
  const [loggingOut, setLoggingOut] = useState(false)

  const handleLogout = () => {
    Effect.runPromise(
      Effect.gen(function* () {
        const auth = yield* Authentication
        setLoggingOut(true)
        yield* auth.logout()
        setLoggingOut(false)
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  return (
    <>
      <PageHeader title="Scope" />
      {user != null && (
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
          <Typography color="text.secondary">Email: {user.email}</Typography>
          <Box>
            <Button
              type="button"
              variant="outlined"
              onClick={handleLogout}
              disabled={loggingOut}
            >
              Log out
            </Button>
          </Box>
        </Box>
      )}
    </>
  )
}

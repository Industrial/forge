import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Link from '@mui/material/Link'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { Effect } from 'effect'
import { Link as RouterLink, useNavigate } from 'react-router-dom'
import { Schema } from 'effect'
import { useCallback, useState } from 'react'
import { useForm, Controller } from 'react-hook-form'

import { Authentication } from '@/features/authentication/services/Authentication'
import { effectSchemaResolver } from '@/lib/effectSchemaResolver'
import { navigateTo } from '@/lib/navigate'
import {
  loginFormSchema,
  type LoginFormValues,
} from '@/schemas/userFormSchemas'
import { getApplicationLayer } from '@/lib/appLayer'
import { AuthenticationError } from '../../errors/AuthenticationError'

export default function LoginPage() {
  const navigate = useNavigate()
  const [submitting, setSubmitting] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const form = useForm<LoginFormValues>({
    resolver: effectSchemaResolver(
      loginFormSchema as Schema.Schema<LoginFormValues, unknown, never>,
    ),
    defaultValues: {
      email: '',
      password: '',
    },
    mode: 'onChange',
  })

  const handleSubmit = useCallback(
    (data: LoginFormValues) => {
      Effect.runPromise(
        Effect.gen(function* () {
          yield* Effect.logDebug('LoginPage.handleSubmit: starting login')
          const auth = yield* Authentication
          setSubmitting(true)
          setErrorMessage(null)
          yield* Effect.logDebug('LoginPage.handleSubmit: calling auth.login')
          yield* auth.login(data.email, data.password)
          yield* Effect.logDebug('LoginPage.handleSubmit: login completed')
          setSubmitting(false)
          yield* Effect.logDebug(
            'LoginPage.handleSubmit: navigating to /dashboard',
          )
          yield* navigateTo(navigate, '/dashboard', { replace: true })
          yield* Effect.logDebug('LoginPage.handleSubmit: navigation completed')
        }).pipe(
          Effect.mapError((error) => {
            setSubmitting(false)
            setErrorMessage(
              error instanceof AuthenticationError
                ? error.message
                : 'Login failed',
            )
          }),
          Effect.provide(getApplicationLayer()),
        ),
      )
    },
    [navigate],
  )

  return (
    <Box data-testid="login-page">
      <Typography variant="h4" component="h1" gutterBottom>
        Log in
      </Typography>
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <form
          onSubmit={form.handleSubmit(handleSubmit)}
          data-testid="login-form"
        >
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {errorMessage != null && (
              <Alert severity="error" data-testid="login-error-message">
                <Typography variant="subtitle2">Login failed</Typography>
                {errorMessage}
              </Alert>
            )}
            <Controller
              control={form.control}
              name="email"
              render={({ field, fieldState }) => (
                <TextField
                  {...field}
                  name="email"
                  type="email"
                  label="Email"
                  placeholder="you@example.com"
                  required
                  slotProps={{
                    htmlInput: {
                      autoComplete: 'email',
                      'data-testid': 'login-email-input',
                    },
                  }}
                  disabled={submitting}
                  fullWidth
                  error={Boolean(fieldState.error)}
                  helperText={fieldState.error?.message}
                />
              )}
            />
            <Controller
              control={form.control}
              name="password"
              render={({ field, fieldState }) => (
                <TextField
                  {...field}
                  name="password"
                  type="password"
                  label="Password"
                  placeholder="••••••••"
                  required
                  slotProps={{
                    htmlInput: {
                      autoComplete: 'current-password',
                      'data-testid': 'login-password-input',
                    },
                  }}
                  disabled={submitting}
                  fullWidth
                  error={Boolean(fieldState.error)}
                  helperText={fieldState.error?.message}
                />
              )}
            />
            <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
              <Button
                type="submit"
                variant="contained"
                disabled={submitting || !form.formState.isValid}
                data-testid="login-submit-button"
              >
                Log in
              </Button>
            </Box>
          </Box>
        </form>
        <Typography variant="body2" textAlign="center">
          <Link
            component={RouterLink}
            to="/authentication/register"
            variant="body2"
            data-testid="login-register-link"
          >
            Don&apos;t have an account? Create an Account
          </Link>
        </Typography>
      </Box>
    </Box>
  )
}

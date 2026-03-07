import { useForm, Controller } from 'react-hook-form'
import { Link as RouterLink, useNavigate } from 'react-router-dom'
import { Schema } from 'effect'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Link from '@mui/material/Link'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { Effect } from 'effect'
import { useCallback, useState } from 'react'

import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { effectSchemaResolver } from '@/lib/effectSchemaResolver'
import { navigateTo } from '@/lib/navigate'
import {
  registerFormSchema,
  type RegisterFormValues,
} from '@/schemas/userFormSchemas'
import { AuthenticationError } from '../../errors/AuthenticationError'

export default function RegisterPage() {
  const navigate = useNavigate()
  const [submitting, setSubmitting] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const form = useForm<RegisterFormValues>({
    resolver: effectSchemaResolver(
      registerFormSchema as Schema.Schema<RegisterFormValues, unknown, never>,
    ),
    defaultValues: { email: '', password: '' },
    mode: 'onChange',
  })

  const handleSubmit = useCallback(
    (data: RegisterFormValues) => {
      Effect.runPromise(
        Effect.gen(function* () {
          const auth = yield* Authentication
          setSubmitting(true)
          setErrorMessage(null)
          yield* auth.register(data.email, data.password)
          setSubmitting(false)
          yield* navigateTo(navigate, '/authentication/login', { replace: true })
        }).pipe(
          Effect.mapError((error) => {
            setSubmitting(false)
            setErrorMessage(
              error instanceof AuthenticationError
                ? error.message
                : 'Registration failed',
            )
          }),
          Effect.provide(getApplicationLayer()),
        ),
      )
    },
    [navigate],
  )

  return (
    <>
      <Typography variant="h4" component="h1" gutterBottom>
        Create an account
      </Typography>
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <form
          onSubmit={form.handleSubmit(handleSubmit)}
          data-testid="register-form"
        >
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {errorMessage != null && (
              <Alert severity="error" data-testid="register-error">
                <Typography variant="subtitle2">Registration failed</Typography>
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
                  slotProps={{ htmlInput: { autoComplete: 'email' } }}
                  disabled={submitting}
                  data-testid="register-email"
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
                  slotProps={{ htmlInput: { minLength: 8, autoComplete: 'new-password' } }}
                  disabled={submitting}
                  data-testid="register-password"
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
                data-testid="register-submit"
              >
                Register
              </Button>
            </Box>
          </Box>
        </form>
        <Typography variant="body2" textAlign="center">
          <Link
            component={RouterLink}
            to="/authentication/login"
            variant="body2"
          >
            Already have an account? Log in
          </Link>
        </Typography>
      </Box>
    </>
  )
}

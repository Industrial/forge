import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Link from '@mui/material/Link'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { Effect } from 'effect'
import { Link as RouterLink, useNavigate } from 'react-router-dom'
import { Schema } from 'effect'
import { useCallback } from 'react'
import { useForm, Controller } from 'react-hook-form'

import { AppLayer } from '@/lib/appLayer'
import { AuthenticationApi } from '@/features/authentication/services/AuthenticationApi'
import { AuthenticationStore } from '@/features/authentication/services/AuthenticationStore'
import { effectSchemaResolver } from '@/lib/effectSchemaResolver'
import {
  loginFormSchema,
  type LoginFormValues,
} from '@/schemas/userFormSchemas'

export default function LoginPage() {
  console.log('LoginPage')

  const navigate = useNavigate()

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
          const api = yield* AuthenticationApi
          const store = yield* AuthenticationStore
          const result = yield* api.login(data.email, data.password)
          yield* store.setToken(result.token)
          yield* store.fetchMe(result.token)
          return result
        }).pipe(Effect.provide(AppLayer)),
      )
    },
    [navigate],
  )

  // const submitting = isPending(submitState)
  const submitting = false
  // const errorMessage =
  //   isFailure(submitState) && submitState.error
  //     ? submitState.error instanceof Error
  //       ? submitState.error.message
  //       : String(submitState.error)
  //     : null
  const errorMessage = null

  return (
    <>
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
              <Alert severity="error" data-testid="login-error">
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
                  disabled={submitting}
                  data-testid="login-email"
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
                  disabled={submitting}
                  data-testid="login-password"
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
                data-testid="login-submit"
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
          >
            Don&apos;t have an account? Create an Account
          </Link>
        </Typography>
      </Box>
    </>
  )
}

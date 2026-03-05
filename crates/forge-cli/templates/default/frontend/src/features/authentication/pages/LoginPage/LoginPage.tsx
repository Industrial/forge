import { useForm, Controller } from 'react-hook-form'
import { Link as RouterLink, useLocation, useNavigate } from 'react-router-dom'
import { Schema } from 'effect'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Link from '@mui/material/Link'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { Effect } from 'effect'
import { useCallback } from 'react'
import {
  AuthenticationApi,
  type LoginResult,
} from '../../services/AuthenticationApi'
import { AuthenticationStore } from '../../services/AuthenticationStore'
import { useAuthentication } from '../../../../context/AuthenticationContext'
import {
  useEffectState,
  useEffectRuntime,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  isPending,
  isFailure,
} from 'react-effect-hooks'
import { runWithAppRuntime, type AppServices } from '../../../../lib/appLayer'
import { effectSchemaResolver } from '../../../../lib/effectSchemaResolver'
import {
  loginFormSchema,
  type LoginFormValues,
} from '../../../../schemas/userFormSchemas'

type LoginState = AsyncState<LoginResult, Error>

export default function LoginPage() {
  const location = useLocation()
  const navigate = useNavigate()
  const { runtime } = useEffectRuntime<AppServices>()
  const { fetchMe } = useAuthentication()

  const from =
    (location.state as { from?: { pathname: string } } | null)?.from
      ?.pathname ?? '/dashboard'

  const [submitState, , setSubmitStateAsEffect] = useEffectState<
    LoginState,
    never,
    never
  >(idle())

  const form = useForm<LoginFormValues>({
    resolver: effectSchemaResolver(
      loginFormSchema as Schema.Schema<LoginFormValues, unknown, never>,
    ),
    defaultValues: { email: '', password: '' },
    mode: 'onChange',
  })

  const loginEffect = useCallback(
    (email: string, password: string): Effect.Effect<LoginResult, Error, AppServices> =>
      Effect.gen(function* () {
        const api = yield* AuthenticationApi
        const store = yield* AuthenticationStore
        const result = yield* api.login(email, password)
        yield* store.setToken(result.token)
        yield* store.fetchMe(result.token)
        return result
      }),
    [],
  )

  const handleSubmit = useCallback(
    (data: LoginFormValues) => {
      const stream = streamWithPendingState(
        loginEffect(data.email, data.password),
      )
      const effect = runStreamInto(stream, setSubmitStateAsEffect)
      runWithAppRuntime(runtime, effect)
        .then(() => fetchMe())
        .then(() => navigate(from, { replace: true }))
        .catch((err) => {
          console.error('Login failed:', err)
        })
    },
    [runtime, loginEffect, setSubmitStateAsEffect, fetchMe, navigate, from],
  )

  const submitting = isPending(submitState)
  const errorMessage =
    isFailure(submitState) && submitState.error
      ? submitState.error instanceof Error
        ? submitState.error.message
        : String(submitState.error)
      : null

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
          <Link component={RouterLink} to="/authentication/register" variant="body2">
            Don&apos;t have an account? Create an Account
          </Link>
        </Typography>
      </Box>
    </>
  )
}

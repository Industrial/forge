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
import { useCallback } from 'react'
import { AuthenticationApi } from '../../services/AuthenticationApi'
import {
  useEffectState,
  useRunEffect,
  useEffectRuntime,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  isSuccess,
  isPending,
  isFailure,
} from '../../../../lib/react-effect'
import { runWithAppRuntime, type AppServices } from '../../../../lib/appLayer'
import { effectSchemaResolver } from '../../../../lib/effectSchemaResolver'
import {
  registerFormSchema,
  type RegisterFormValues,
} from '../../../../schemas/userFormSchemas'

type RegisterState = AsyncState<void, Error>

export default function RegisterPage() {
  const navigate = useNavigate()
  const { runtime } = useEffectRuntime<AppServices>()

  const [submitState, , setSubmitStateAsEffect] = useEffectState<
    RegisterState,
    never,
    never
  >(idle())

  const form = useForm<RegisterFormValues>({
    resolver: effectSchemaResolver(
      registerFormSchema as Schema.Schema<RegisterFormValues, unknown, never>,
    ),
    defaultValues: { email: '', password: '' },
    mode: 'onChange',
  })

  const registerEffect = useCallback(
    (email: string, password: string): Effect.Effect<void, Error, AppServices> =>
      Effect.gen(function* () {
        const api = yield* AuthenticationApi
        yield* api.register(email, password)
      }),
    [],
  )

  const handleSubmit = useCallback(
    (data: RegisterFormValues) => {
      const stream = streamWithPendingState(
        registerEffect(data.email, data.password),
      )
      const effect = runStreamInto(stream, setSubmitStateAsEffect)
      runWithAppRuntime(runtime, effect).catch(() => {})
    },
    [runtime, registerEffect, setSubmitStateAsEffect],
  )

  const successEffect: Effect.Effect<void, never, AppServices> = Effect.gen(
    function* () {
      if (!isSuccess(submitState)) return
      yield* Effect.sync(() => navigate('/authentication/login', { replace: true }))
      yield* setSubmitStateAsEffect(idle<void, Error>())
    },
  )
  useRunEffect(successEffect, [submitState, navigate, setSubmitStateAsEffect])

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
                  inputProps={{ minLength: 8 }}
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
          <Link component={RouterLink} to="/authentication/login" variant="body2">
            Already have an account? Log in
          </Link>
        </Typography>
      </Box>
    </>
  )
}

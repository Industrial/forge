import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Link from '@mui/material/Link'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { Effect, Either } from 'effect'
import { Link as RouterLink, useNavigate } from 'react-router-dom'
import { useCallback, useState } from 'react'

import { Authentication } from '@/features/authentication/services/Authentication'
import { navigateTo } from '@/lib/navigate'
import { getApplicationLayer } from '@/lib/appLayer'
import { AuthenticationError } from '@/features/authentication/errors/AuthenticationError'
import { useForm } from '@/hooks/useForm'
import { LoginFormSchema } from '@/features/authentication/schemas/LoginFormSchema'

export default function LoginPage() {
  const navigate = useNavigate()
  const [submitting, setSubmitting] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const {
    formState,
    setFieldValue,
    setFieldTouched,
    getFieldError,
    validateForm,
    setValidationErrors,
  } = useForm({
    schema: LoginFormSchema,
    initialValues: {
      email: '',
      password: '',
    },
  })

  /**
   * Effect.ts-based submit handler.
   * Validates form, then logs in user and navigates on success.
   * Error handling is done within the Effect pipeline.
   */
  const handleSubmit = useCallback(
    (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()

      // Validate immediately before submit to show errors and prevent invalid submission
      const currentValues = formState.values
      const validationEffect = validateForm(currentValues)
      const validationResult = Effect.runSync(
        validationEffect.pipe(
          Effect.either,
          Effect.map((either) =>
            Either.match(either, {
              onLeft: (errors) => ({ errors, isValid: false }),
              onRight: () => ({ errors: [], isValid: true }),
            }),
          ),
        ),
      )

      // Mark all fields as touched and set validation errors
      // This ensures errors are displayed for all fields
      setValidationErrors(validationResult.errors)

      // Don't submit if validation failed
      // Validation errors are shown via field-level helperText, not top-level Alert
      if (!validationResult.isValid) {
        return
      }

      const submitEffect = Effect.gen(function* () {
        // Validate form values (should succeed since we validated above)
        const validated = yield* validateForm(currentValues)

        // Login logic
        const auth = yield* Authentication
        yield* auth.login(validated.email, validated.password)

        // Navigate on success
        yield* navigateTo(navigate, '/dashboard', { replace: true })

        // Return success result
        return { type: 'success' as const }
      }).pipe(
        Effect.catchAll((error) => {
          return Effect.gen(function* () {
            // Handle validation errors
            if (Array.isArray(error)) {
              return yield* Effect.succeed({
                type: 'validation' as const,
                errors: error,
              })
            }

            // Handle authentication errors
            if (error instanceof AuthenticationError) {
              return yield* Effect.succeed({
                type: 'authentication' as const,
                message: error.message,
              })
            }

            // Handle other errors
            return yield* Effect.succeed({
              type: 'unknown' as const,
              message: 'Login failed',
            })
          })
        }),
        Effect.provide(getApplicationLayer()),
      )

      setSubmitting(true)
      setErrorMessage(null)

      Effect.runPromise(submitEffect).then((result) => {
        setSubmitting(false)

        if (result.type === 'success') {
          // Success - navigation already happened, nothing to do
          return
        }

        if (result.type === 'validation') {
          // Update form state with validation errors
          // Validation errors are shown via field-level helperText, not top-level Alert
          setValidationErrors(result.errors)
        } else if (
          result.type === 'authentication' ||
          result.type === 'unknown'
        ) {
          // Only show top-level Alert for backend/authentication errors
          setErrorMessage(result.message)
        }
      })
    },
    [navigate, formState.values, validateForm, setValidationErrors],
  )

  const emailError = getFieldError('email')
  const passwordError = getFieldError('password')

  return (
    <Box data-testid="login-page">
      <Typography variant="h4" component="h1" gutterBottom>
        Log in
      </Typography>
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <form onSubmit={handleSubmit} data-testid="login-form">
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {errorMessage != null && (
              <Alert severity="error" data-testid="login-error-message">
                <Typography variant="subtitle2">Login failed</Typography>
                {errorMessage}
              </Alert>
            )}
            <TextField
              name="email"
              type="text"
              label="Email"
              placeholder="you@example.com"
              value={formState.values.email}
              onChange={(e) => setFieldValue('email', e.target.value)}
              onBlur={() => setFieldTouched('email')}
              data-testid="login-email"
              slotProps={{
                htmlInput: {
                  autoComplete: 'email',
                  'data-testid': 'login-email-input',
                },
              }}
              disabled={submitting}
              fullWidth
              error={Boolean(emailError)}
              helperText={emailError}
            />
            <TextField
              name="password"
              type="password"
              label="Password"
              placeholder="••••••••"
              value={formState.values.password}
              onChange={(e) => setFieldValue('password', e.target.value)}
              onBlur={() => setFieldTouched('password')}
              data-testid="login-password"
              slotProps={{
                htmlInput: {
                  autoComplete: 'current-password',
                  'data-testid': 'login-password-input',
                },
              }}
              disabled={submitting}
              fullWidth
              error={Boolean(passwordError)}
              helperText={passwordError}
            />
            <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
              <Button
                type="submit"
                variant="contained"
                disabled={submitting}
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

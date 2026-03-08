import { Link as RouterLink, useNavigate } from 'react-router-dom'
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
import { navigateTo } from '@/lib/navigate'
import { AuthenticationError } from '@/features/authentication/errors/AuthenticationError'
import { useRegisterForm } from '@/features/authentication/hooks/useRegisterForm'

export default function RegisterPage() {
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
  } = useRegisterForm()

  /**
   * Effect.ts-based submit handler.
   * Validates form, then registers user and navigates on success.
   * Error handling is done within the Effect pipeline.
   */
  const handleSubmit = useCallback(
    (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()

      // Mark all fields as touched so errors show
      setFieldTouched('email')
      setFieldTouched('password')

      const submitEffect = Effect.gen(function* () {
        // Validate form values
        const validated = yield* validateForm(formState.values)

        // Registration logic
        const auth = yield* Authentication
        yield* auth.register(validated.email, validated.password)

        // Navigate on success
        yield* navigateTo(navigate, '/authentication/login', {
          replace: true,
        })
      }).pipe(
        Effect.catchAll((error) => {
          return Effect.gen(function* () {
            // Handle validation errors
            if (Array.isArray(error)) {
              // Update form state with validation errors
              // This will be handled by updating formState via setFieldTouched
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
              message: 'Registration failed',
            })
          })
        }),
        Effect.provide(getApplicationLayer()),
      )

      setSubmitting(true)
      setErrorMessage(null)

      Effect.runPromise(submitEffect).then((result) => {
        setSubmitting(false)

        if (result.type === 'validation') {
          // Update form state with validation errors
          setValidationErrors(result.errors)
          // Show top-level validation error message
          setErrorMessage('Please correct the errors in the form')
        } else if (
          result.type === 'authentication' ||
          result.type === 'unknown'
        ) {
          setErrorMessage(result.message)
        }
      })
    },
    [
      navigate,
      formState.values,
      validateForm,
      setFieldTouched,
      setValidationErrors,
    ],
  )

  const emailError = getFieldError('email')
  const passwordError = getFieldError('password')
  // Only show top-level error for authentication errors or validation errors on submit attempt
  // Field-level errors are shown via helperText, not the top-level Alert
  const showTopLevelError = errorMessage != null

  return (
    <Box data-testid="register-page">
      <Typography variant="h4" component="h1" gutterBottom>
        Create an account
      </Typography>
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <form onSubmit={handleSubmit} data-testid="register-form">
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {showTopLevelError && (
              <Alert severity="error" data-testid="register-error-message">
                <Typography variant="subtitle2">
                  {errorMessage != null
                    ? 'Registration failed'
                    : 'Validation error'}
                </Typography>
                {errorMessage != null
                  ? errorMessage
                  : 'Please correct the errors in the form'}
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
              slotProps={{
                htmlInput: {
                  autoComplete: 'email',
                  'data-testid': 'register-email-input',
                },
              }}
              disabled={submitting}
              fullWidth
              required
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
              slotProps={{
                htmlInput: {
                  autoComplete: 'new-password',
                  'data-testid': 'register-password-input',
                },
              }}
              disabled={submitting}
              fullWidth
              required
              error={Boolean(passwordError)}
              helperText={passwordError}
            />
            <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
              <Button
                type="submit"
                variant="contained"
                disabled={submitting}
                data-testid="register-submit-button"
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
            data-testid="register-login-link"
          >
            Already have an account? Log in
          </Link>
        </Typography>
      </Box>
    </Box>
  )
}

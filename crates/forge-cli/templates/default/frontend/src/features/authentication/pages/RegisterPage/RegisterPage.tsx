import { Link as RouterLink, useNavigate } from 'react-router-dom'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Link from '@mui/material/Link'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { Effect, Either } from 'effect'
import { useCallback, useState } from 'react'

import { getApplicationLayer } from '../../../../lib/appLayer'
import { Authentication } from '../../../../features/authentication/services/Authentication'
import { navigateTo } from '../../../../lib/navigate'
import { AuthenticationError } from '../../../../features/authentication/errors/AuthenticationError'
import { useForm } from '../../../../hooks/useForm'
import { RegisterFormSchema } from '../../../../features/authentication/schemas/RegisterFormSchema'

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
  } = useForm({
    schema: RegisterFormSchema,
    initialValues: {
      email: '',
      password: '',
    },
  })

  /**
   * Effect.ts-based submit handler.
   * Validates form, then registers user and navigates on success.
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

        // Registration logic
        const auth = yield* Authentication
        yield* auth.register(validated.email, validated.password)

        // Navigate on success
        yield* navigateTo(navigate, '/authentication/login', {
          replace: true,
        })

        // Return success result
        return { type: 'success' as const }
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
              data-testid="register-email"
              slotProps={{
                htmlInput: {
                  autoComplete: 'email',
                  'data-testid': 'register-email-input',
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
              data-testid="register-password"
              slotProps={{
                htmlInput: {
                  autoComplete: 'new-password',
                  'data-testid': 'register-password-input',
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

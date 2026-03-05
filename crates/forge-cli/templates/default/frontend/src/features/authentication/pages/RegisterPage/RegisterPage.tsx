import { useState } from 'react'
import { useForm, Controller } from 'react-hook-form'
import { Link as RouterLink, useNavigate } from 'react-router-dom'
import { Schema } from 'effect'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Link from '@mui/material/Link'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { effectSchemaResolver } from '../../../../lib/effectSchemaResolver'
import {
  registerFormSchema,
  type RegisterFormValues,
} from '../../../../schemas/userFormSchemas'

export default function RegisterPage() {
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  const form = useForm<RegisterFormValues>({
    resolver: effectSchemaResolver(
      registerFormSchema as Schema.Schema<RegisterFormValues, unknown, never>,
    ),
    defaultValues: { email: '', password: '' },
    mode: 'onChange',
  })

  async function handleSubmit(data: RegisterFormValues) {
    setSubmitting(true)
    setError(null)
    try {
      const res = await fetch('/api/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ email: data.email, password: data.password }),
      })
      const resData = await res.json().catch(() => ({}))
      if (res.ok) {
        navigate('/login', { replace: true })
        return
      }
      setError(
        typeof resData?.error === 'string'
          ? resData.error
          : 'Registration failed. Please try again.',
      )
    } catch {
      setError('Something went wrong. Please try again.')
    } finally {
      setSubmitting(false)
    }
  }

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
            {error != null && (
              <Alert severity="error" data-testid="register-error">
                <Typography variant="subtitle2">Registration failed</Typography>
                {error}
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
          <Link component={RouterLink} to="/login" variant="body2">
            Already have an account? Log in
          </Link>
        </Typography>
      </Box>
    </>
  )
}

import { RegisterFormSchema } from '@/features/authentication/schemas/RegisterFormSchema'
import { useForm, type FieldError } from '@/features/authentication/hooks/useForm'
import type { RegisterFormValues } from '@/features/authentication/schemas/RegisterFormSchema'

/**
 * Effect.ts-based form hook for registration form.
 * Uses the generalized useForm hook with RegisterFormSchema.
 */
export function useRegisterForm() {
  return useForm<RegisterFormValues>({
    schema: RegisterFormSchema,
    initialValues: {
      email: '',
      password: '',
    },
  })
}

// Re-export types for backward compatibility
export type { FieldError }

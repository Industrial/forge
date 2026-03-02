import { Button, Form, Heading, Link, TextField } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import { router } from '@inertiajs/react';
import AuthLayout from '../../components/AuthLayout';

export default function Register() {
  return (
    <AuthLayout>
      <Heading level={1}>Create an account</Heading>
      <Form
        onSubmit={(e) => {
          e.preventDefault();
          const formData = new FormData(e.currentTarget);
          const email = (formData.get('email') ?? '') as string;
          const password = (formData.get('password') ?? '') as string;
          router.post('/api/auth/register', { email, password });
        }}
      >
        <div className={style({ display: 'flex', flexDirection: 'column', gap: 16 })}>
          <TextField
            name="email"
            type="email"
            label="Email"
            placeholder="you@example.com"
            isRequired
          />
          <TextField
            name="password"
            type="password"
            label="Password"
            placeholder="••••••••"
            isRequired
          />
          <Button type="submit" variant="accent">
            Register
          </Button>
        </div>
      </Form>
      <div className={style({ display: 'flex', gap: 16, flexWrap: 'wrap' })}>
        <Link href="/login">Already have an account? Log in</Link>
        <Link href="/">Back to home</Link>
      </div>
    </AuthLayout>
  );
}

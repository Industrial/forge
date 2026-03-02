import { Button, Heading, Link } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import AuthLayout from '../../components/AuthLayout';

export default function Login() {
  return (
    <AuthLayout>
      <Heading level={1}>Log in</Heading>
      {/* Native form with target="_top" so the redirect after login is a full
          document load and the session cookie is sent on GET /dashboard. */}
      <form
        action="/api/auth/login"
        method="post"
        target="_top"
        className={style({ display: 'flex', flexDirection: 'column', gap: 16 })}
      >
        <label className={style({ font: 'body', display: 'flex', flexDirection: 'column', gap: 4 })}>
          Email
          <input
            name="email"
            type="email"
            placeholder="you@example.com"
            required
            style={{
              padding: '8px 12px',
              borderRadius: 6,
              border: '1px solid var(--spectrum-gray-300)',
              fontSize: 14,
            }}
          />
        </label>
        <label className={style({ font: 'body', display: 'flex', flexDirection: 'column', gap: 4 })}>
          Password
          <input
            name="password"
            type="password"
            placeholder="••••••••"
            required
            style={{
              padding: '8px 12px',
              borderRadius: 6,
              border: '1px solid var(--spectrum-gray-300)',
              fontSize: 14,
            }}
          />
        </label>
        <Button type="submit" variant="accent">
          Log in
        </Button>
      </form>
      <div className={style({ display: 'flex', gap: 16, flexWrap: 'wrap' })}>
        <Link href="/register">Create an account</Link>
        <Link href="/">Back to home</Link>
      </div>
    </AuthLayout>
  );
}

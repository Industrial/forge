import { Button, Heading, Link } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';

export default function Register() {
  return (
    <>
      <Heading level={1}>Create an account</Heading>
      <form
        action="/api/auth/register"
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
            minLength={8}
            style={{
              padding: '8px 12px',
              borderRadius: 6,
              border: '1px solid var(--spectrum-gray-300)',
              fontSize: 14,
            }}
          />
        </label>
        <Button type="submit" variant="accent">
          Register
        </Button>
      </form>
      <div className={style({ display: 'flex', gap: 16, flexWrap: 'wrap' })}>
        <Link href="/login" isQuiet>Already have an account? Log in</Link>
        <Link href="/" isQuiet>Back to home</Link>
      </div>
    </>
  );
}

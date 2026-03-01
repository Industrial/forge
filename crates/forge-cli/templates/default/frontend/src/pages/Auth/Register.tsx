import React from 'react';
import { router } from '@inertiajs/react';

export default function Register() {
  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const email = (form.elements.namedItem('email') as HTMLInputElement | null)?.value ?? '';
    const password = (form.elements.namedItem('password') as HTMLInputElement | null)?.value ?? '';
    router.post('/api/auth/register', { email, password });
  };

  return (
    <div>
      <h1>Register</h1>
      <form action="/api/auth/register" method="post" onSubmit={submit}>
        <input name="email" type="email" placeholder="Email" required />
        <input name="password" type="password" placeholder="Password" required />
        <button type="submit">Register</button>
      </form>
      <a href="/login">Login</a> | <a href="/">Home</a>
    </div>
  );
}

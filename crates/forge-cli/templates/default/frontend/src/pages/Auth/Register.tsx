import React, { useState } from 'react';
import { router } from '@inertiajs/react';

export default function Register() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    router.post('/api/auth/register', { email, password });
  };

  return (
    <div>
      <h1>Register</h1>
      <form onSubmit={submit}>
        <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder="Email" required />
        <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder="Password" required />
        <button type="submit">Register</button>
      </form>
      <a href="/login">Login</a> | <a href="/">Home</a>
    </div>
  );
}

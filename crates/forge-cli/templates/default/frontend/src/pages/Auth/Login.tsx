import React from 'react';

export default function Login() {
  return (
    <div>
      <h1>Login</h1>
      {/* Native form submit with target="_top" so the redirect after login is a full
          document load and the session cookie is sent on GET /dashboard. */}
      <form
        action="/api/auth/login"
        method="post"
        target="_top"
      >
        <input name="email" type="email" placeholder="Email" required />
        <input name="password" type="password" placeholder="Password" required />
        <button type="submit">Log in</button>
      </form>
      <a href="/register">Register</a> | <a href="/">Home</a>
    </div>
  );
}

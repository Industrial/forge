import React from 'react';

type Props = { locale: string; greeting: string };

export default function Home({ locale, greeting }: Props) {
  return (
    <div>
      <h1>{greeting}</h1>
      <p>Locale: {locale}</p>
      <nav>
        <a href="/login">Login</a> | <a href="/register">Register</a> | <a href="/dashboard">Dashboard</a> | <a href="/ws-demo">WebSocket</a>
      </nav>
    </div>
  );
}
